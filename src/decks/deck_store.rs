use super::Deck;
use super::DeckEntry;
use super::DeckSummary;
use crate::db::Database;
use crate::model::Card;
use crate::model::DeckCard;
use eyre::Context;
use eyre::ContextCompat;
use eyre::Result;
use eyre::ensure;
use std::fs::File;
use std::fs::OpenOptions;
use std::fs::{self};
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
#[derive(Debug)]
pub struct DeckStore {
    dir: PathBuf,
}
impl DeckStore {
    pub fn new(dir: &Path) -> Result<Self> {
        let dir = dir.join("decks");
        fs::create_dir_all(&dir)?;
        Ok(Self { dir })
    }
    fn validate_name(name: &str) -> Result<()> {
        ensure!(
            !name.is_empty()
                && name.len() <= 100
                && name == name.trim()
                && !name.ends_with('.')
                && !name
                    .chars()
                    .any(|c| c.is_control() || r#"<>:"/\|?*"#.contains(c)),
            "Deck name must be 1-100 bytes and a valid filename (no slashes or surrounding whitespace)"
        );
        let stem = name.split('.').next().unwrap_or("").to_uppercase();
        ensure!(
            ![
                "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7",
                "COM8", "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8",
                "LPT9"
            ]
            .contains(&stem.as_str()),
            "Reserved deck filename"
        );
        Ok(())
    }
    fn path(&self, name: &str) -> Result<PathBuf> {
        Self::validate_name(name)?;
        // Case-insensitive names behave consistently on Windows and Unix.
        for entry in fs::read_dir(&self.dir)? {
            let path = entry?.path();
            if path.extension().is_some_and(|e| e == "json")
                && path
                    .file_stem()
                    .is_some_and(|s| s.to_string_lossy().to_lowercase() == name.to_lowercase())
            {
                return Ok(path);
            }
        }
        Ok(self.dir.join(format!("{name}.json")))
    }
    fn lock(&self) -> Result<File> {
        let file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(self.dir.join(".lock"))?;
        file.lock()?;
        Ok(file)
    }
    pub fn load(&self, name: &str) -> Result<Deck> {
        let path = self.path(name)?;
        let text = fs::read_to_string(&path).with_context(|| {
            format!(
                "Cannot read deck '{}'; create it with 'deck create {name}'",
                path.display()
            )
        })?;
        let deck: Deck = facet_json::from_str(&text)
            .with_context(|| format!("Invalid deck JSON in {}", path.display()))?;
        ensure!(
            deck.version == 1,
            "Unsupported deck version {}",
            deck.version
        );
        ensure!(
            deck.name.to_lowercase() == name.to_lowercase(),
            "Deck name does not match filename"
        );
        let mut ids = std::collections::HashSet::new();
        for entry in &deck.cards {
            ensure!(
                !entry.id.is_empty()
                    && !entry.name.is_empty()
                    && (1..=10000).contains(&entry.quantity),
                "Invalid deck entry in {}",
                path.display()
            );
            ensure!(ids.insert(&entry.id), "Duplicate printing ID in deck JSON");
        }
        Ok(deck)
    }
    fn save(&self, deck: &Deck, create: bool) -> Result<()> {
        let path = self.path(&deck.name)?;
        let mut temp = tempfile::NamedTempFile::new_in(&self.dir)?;
        writeln!(temp, "{}", facet_json::to_string_pretty(deck)?)?;
        temp.as_file().sync_all()?;
        if create {
            temp.persist_noclobber(path)?;
        } else {
            temp.persist(path)?;
        }
        Ok(())
    }
    pub fn create(&self, name: &str) -> Result<Deck> {
        let _lock = self.lock()?;
        ensure!(!self.path(name)?.exists(), "Deck '{name}' already exists");
        let deck = Deck {
            version: 1,
            name: name.into(),
            cards: Vec::new(),
        };
        self.save(&deck, true)?;
        Ok(deck)
    }
    pub fn list(&self) -> Result<Vec<DeckSummary>> {
        let mut decks = Vec::new();
        for entry in fs::read_dir(&self.dir)? {
            let path = entry?.path();
            if path.extension().is_none_or(|e| e != "json") {
                continue;
            }
            let name = path
                .file_stem()
                .context("Invalid deck filename")?
                .to_string_lossy();
            let deck = self.load(&name)?;
            decks.push(DeckSummary {
                name: deck.name,
                total_cards: deck.cards.iter().map(|c| u64::from(c.quantity)).sum(),
                unique_cards: deck.cards.len(),
                path: path.display().to_string(),
            });
        }
        decks.sort_by_key(|d| d.name.to_lowercase());
        Ok(decks)
    }
    pub fn add(&self, deck: &str, card: &Card, quantity: u32) -> Result<Deck> {
        ensure!(
            (1..=10000).contains(&quantity),
            "Quantity must be between 1 and 10000"
        );
        let _lock = self.lock()?;
        let mut deck = self.load(deck)?;
        if let Some(entry) = deck.cards.iter_mut().find(|c| c.id == card.id) {
            let total = entry
                .quantity
                .checked_add(quantity)
                .context("Quantity overflow")?;
            ensure!(total <= 10000, "Maximum 10000 copies per printing");
            entry.quantity = total;
        } else {
            deck.cards.push(DeckEntry {
                id: card.id.clone(),
                name: card.name.clone(),
                quantity,
            });
        }
        deck.cards.sort_by(|a, b| {
            a.name
                .to_lowercase()
                .cmp(&b.name.to_lowercase())
                .then(a.id.cmp(&b.id))
        });
        self.save(&deck, false)?;
        Ok(deck)
    }
    pub fn remove(&self, name: &str, card: &str, quantity: Option<u32>) -> Result<Deck> {
        ensure!(quantity != Some(0), "Quantity must be positive");
        let _lock = self.lock()?;
        let mut deck = self.load(name)?;
        let matches: Vec<_> = deck
            .cards
            .iter()
            .enumerate()
            .filter(|(_, c)| c.id == card || c.name.to_lowercase() == card.to_lowercase())
            .map(|(i, _)| i)
            .collect();
        ensure!(
            matches.len() == 1,
            "Expected one matching deck entry for '{card}', found {}; use a printing ID if ambiguous",
            matches.len()
        );
        let i = matches[0];
        let remove = quantity.unwrap_or(deck.cards[i].quantity);
        ensure!(
            remove <= deck.cards[i].quantity,
            "Deck has only {} copies",
            deck.cards[i].quantity
        );
        deck.cards[i].quantity -= remove;
        deck.cards.retain(|c| c.quantity > 0);
        self.save(&deck, false)?;
        Ok(deck)
    }
    pub fn resolve(&self, name: &str, db: &Database) -> Result<Vec<DeckCard>> {
        self.load(name)?
            .cards
            .into_iter()
            .map(|c| {
                Ok(DeckCard {
                    quantity: c.quantity,
                    card: db.resolve_card(&c.id).with_context(|| {
                        format!(
                            "Deck card '{}' ({}) is missing from the catalog; import its card data",
                            c.name, c.id
                        )
                    })?,
                })
            })
            .collect()
    }
}
