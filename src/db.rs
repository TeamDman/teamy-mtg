use crate::model::Card;
use crate::search::Query;
use eyre::Context;
use eyre::ContextCompat;
use eyre::Result;
use eyre::bail;
use eyre::ensure;
use rusqlite::Connection;
use rusqlite::OptionalExtension;
use rusqlite::Transaction;
use rusqlite::params;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Read;
use std::path::Path;
use std::time::Duration;

#[derive(Debug)]
pub struct Database {
    conn: Connection,
    cancellation: Option<teamy_cancellation::CancellationToken>,
}
impl Database {
    pub fn open(dir: &Path) -> Result<Self> {
        std::fs::create_dir_all(dir)
            .with_context(|| format!("Cannot create data directory {}", dir.display()))?;
        let conn = Connection::open(dir.join("teamy-mtg.sqlite3"))?;
        conn.busy_timeout(Duration::from_secs(30))?;
        let version: u32 = conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;
        ensure!(
            version <= 1,
            "Database schema {version} is newer than this CLI supports"
        );
        conn.execute_batch(
            "BEGIN IMMEDIATE;
            CREATE TABLE IF NOT EXISTS cards (
                id TEXT PRIMARY KEY, oracle_id TEXT NOT NULL, name TEXT NOT NULL,
                name_fold TEXT NOT NULL, data TEXT NOT NULL, active INTEGER NOT NULL DEFAULT 1
            );
            CREATE INDEX IF NOT EXISTS cards_name ON cards(name_fold);
            CREATE TABLE IF NOT EXISTS metadata (key TEXT PRIMARY KEY, value TEXT NOT NULL);
            PRAGMA user_version=1;
            COMMIT;",
        )?;
        Ok(Self {
            conn,
            cancellation: None,
        })
    }
    pub fn set_cancellation(&mut self, token: teamy_cancellation::CancellationToken) {
        self.cancellation = Some(token);
    }
    pub fn import_file(&mut self, path: &Path, source: &str) -> Result<usize> {
        let mut reader = BufReader::new(
            File::open(path).with_context(|| format!("Cannot open {}", path.display()))?,
        );
        if reader.fill_buf()?.starts_with(&[0x1f, 0x8b]) {
            self.import_reader(
                BufReader::new(flate2::read::MultiGzDecoder::new(reader)),
                source,
            )
        } else {
            self.import_reader(reader, source)
        }
    }
    pub fn import_reader(&mut self, reader: impl BufRead, source: &str) -> Result<usize> {
        let cancellation = self.cancellation.clone();
        let tx = self.conn.transaction()?;
        tx.execute("UPDATE cards SET active=0", [])?;
        let count = read_cards(reader, |raw| {
            if let Some(token) = &cancellation {
                token.bail_if_cancelled()?;
            }
            let card: Card =
                facet_json::from_str(raw).context("Invalid card JSON (update rolled back)")?;
            insert_card(&tx, &card, raw)
        })?;
        ensure!(
            count > 0,
            "Card data contains no cards (update rolled back)"
        );
        // Old printings remain addressable by ID: portable JSON decks can reference them.
        tx.execute(
            "INSERT OR REPLACE INTO metadata(key,value) VALUES ('source',?1)",
            [source],
        )?;
        tx.execute("INSERT OR REPLACE INTO metadata(key,value) VALUES ('updated_at',strftime('%Y-%m-%dT%H:%M:%SZ','now'))", [])?;
        tx.commit()?;
        Ok(count)
    }
    pub fn list_cards(&self, query: Option<&str>, limit: u32, offset: u32) -> Result<Vec<Card>> {
        let mut stmt = self.conn.prepare("SELECT data FROM cards WHERE active=1 AND instr(name_fold,?1)>0 ORDER BY name_fold,id LIMIT ?2 OFFSET ?3")?;
        let rows = stmt.query_map(
            params![query.unwrap_or("").to_lowercase(), limit, offset],
            |r| r.get::<_, String>(0),
        )?;
        rows.map(|r| Ok(facet_json::from_str(&r?)?)).collect()
    }
    pub fn search(&self, query: &Query, limit: u32, offset: u32) -> Result<Vec<Card>> {
        let mut stmt = self
            .conn
            .prepare("SELECT data FROM cards WHERE active=1 ORDER BY name_fold,id")?;
        let rows = stmt.query_map([], |r| r.get::<_, String>(0))?;
        let mut cards = Vec::new();
        let mut skip = offset;
        for raw in rows {
            if let Some(token) = &self.cancellation {
                token.bail_if_cancelled()?;
            }
            let card: Card = facet_json::from_str(&raw?)?;
            if !query.matches(&card) {
                continue;
            }
            if skip > 0 {
                skip -= 1;
                continue;
            }
            cards.push(card);
            if cards.len() >= limit as usize {
                break;
            }
        }
        Ok(cards)
    }
    pub fn resolve_card(&self, name: &str) -> Result<Card> {
        if let Some(data) = self
            .conn
            .query_row("SELECT data FROM cards WHERE id=?1", [name], |r| {
                r.get::<_, String>(0)
            })
            .optional()?
        {
            return Ok(facet_json::from_str(&data)?);
        }
        let mut stmt = self
            .conn
            .prepare("SELECT data FROM cards WHERE name_fold=?1 ORDER BY active DESC,id")?;
        let cards: Vec<Card> = stmt
            .query_map([name.to_lowercase()], |r| r.get::<_, String>(0))?
            .map(|r| Ok(facet_json::from_str(&r?)?))
            .collect::<Result<_>>()?;
        let Some(card) = cards.first() else {
            bail!(
                "Card '{name}' was not found locally. Run 'teamy-mtg db update' first; use an exact full name or printing ID"
            );
        };
        ensure!(
            cards.iter().all(|c| c.oracle_id == card.oracle_id),
            "Multiple distinct cards named '{name}'; use a printing ID"
        );
        Ok(card.clone())
    }
}
fn insert_card(tx: &Transaction<'_>, card: &Card, raw: &str) -> Result<()> {
    card.validate()?;
    tx.prepare_cached("INSERT INTO cards(id,oracle_id,name,name_fold,data,active) VALUES (?1,?2,?3,?4,?5,1) ON CONFLICT(id) DO UPDATE SET oracle_id=excluded.oracle_id,name=excluded.name,name_fold=excluded.name_fold,data=excluded.data,active=1")?
        .execute(params![card.id, card.oracle_id, card.name, card.name.to_lowercase(), raw])?;
    Ok(())
}

/// Frame one object at a time; Facet owns all JSON decoding and validation.
/// This keeps bulk arrays and JSONL imports bounded to one card's data.
fn read_cards(reader: impl Read, mut insert: impl FnMut(&str) -> Result<()>) -> Result<usize> {
    let mut bytes = BufReader::new(reader).bytes();
    fn next(bytes: &mut impl Iterator<Item = std::io::Result<u8>>) -> Result<Option<u8>> {
        for b in bytes {
            let b = b?;
            if !b.is_ascii_whitespace() {
                return Ok(Some(b));
            }
        }
        Ok(None)
    }
    let first = next(&mut bytes)?.context("Card data is empty")?;
    let array = first == b'[';
    let mut start = if array {
        next(&mut bytes)?
    } else {
        Some(first)
    };
    if array && start == Some(b']') {
        ensure!(next(&mut bytes)?.is_none(), "Unexpected trailing data");
        return Ok(0);
    }
    let mut count = 0;
    loop {
        ensure!(
            start == Some(b'{'),
            "Expected a card object (update rolled back)"
        );
        let mut raw = vec![b'{'];
        let (mut depth, mut quoted, mut escaped) = (1usize, false, false);
        while depth > 0 {
            let b = bytes.next().context("Truncated card JSON")??;
            raw.push(b);
            ensure!(raw.len() <= 4 * 1024 * 1024, "Card record exceeds 4 MiB");
            if quoted {
                if escaped {
                    escaped = false;
                } else if b == b'\\' {
                    escaped = true;
                } else if b == b'"' {
                    quoted = false;
                }
            } else {
                match b {
                    b'"' => quoted = true,
                    b'{' => depth += 1,
                    b'}' => depth -= 1,
                    _ => {}
                }
            }
        }
        insert(std::str::from_utf8(&raw).context("Card JSON is not UTF-8")?)?;
        count += 1;
        if count % 10000 == 0 {
            tracing::info!(count, "Imported cards");
        }
        let separator = next(&mut bytes)?;
        if array {
            match separator {
                Some(b',') => start = next(&mut bytes)?,
                Some(b']') => {
                    ensure!(
                        next(&mut bytes)?.is_none(),
                        "Unexpected content after card array"
                    );
                    break;
                }
                _ => bail!("Expected comma or closing array bracket"),
            }
        } else if separator.is_none() {
            break;
        } else {
            start = separator;
        }
    }
    Ok(count)
}
