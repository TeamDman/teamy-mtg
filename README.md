# Teamy MTG

An offline Magic card catalog, JSON deck manager, image cache, and proxy PDF generator.
Built from the `teamy-rust-cli` initializer: Facet/Figue arguments, Facet JSON,
structured output, logging, cancellation, build metadata, and Windows resources.
Application code does not use Serde or clap. CLI subcommands have their own
directory and descriptive `*_cli.rs` file; types live in separate files.

## Install and start

Requires Rust 1.96 or newer and the platform's Rust build tools. SQLite is bundled.

```powershell
cargo install --path . --locked

teamy-mtg db update
teamy-mtg card list
teamy-mtg card search "id:g is:commander"
teamy-mtg deck list
teamy-mtg deck create my-deck-1
teamy-mtg deck card add --deck my-deck-1 "Zulaport Cutthroat"
teamy-mtg deck card list --deck my-deck-1
teamy-mtg card image list --deck my-deck-1
teamy-mtg card image sync --deck my-deck-1
teamy-mtg proxy-pdf generate --deck my-deck-1 a.pdf --offline
```

To build without installing, use `cargo build --release`, then
`./target/release/teamy-mtg.exe` on Windows or `./target/release/teamy-mtg` on Unix.
Run `--help` at any command level. Global options work before or after subcommands.

## Offline data

`db update` downloads the complete [Scryfall Oracle Cards bulk catalog](https://scryfall.com/docs/api/bulk-data):
one representative printing per Oracle ID, including multi-face cards. Searches,
card listing, and deck operations then work without a network connection.
It is the complete Oracle catalog, **not every printing of every card**. Printing-specific
filters inspect the selected printings in your local catalog.

You can transfer an export to an offline machine or import a larger Scryfall
printing dataset yourself:

```powershell
teamy-mtg db update --from oracle-cards.jsonl.gz
teamy-mtg db update --from default-cards.json
```

JSON arrays and JSONL are accepted, with optional gzip compression. Imports stream
one card at a time and commit in one transaction. Invalid/truncated data rolls back.
Refreshes retain old printings by ID so existing decks keep their selected artwork.
Old retained printings are omitted from catalog search/list results.

`--data-dir PATH` overrides the data directory; otherwise `TEAMY_MTG_DATA_DIR`
or the template's OS application-config location is used. `home show` reports it.

```text
<data directory>/
  teamy-mtg.sqlite3
  decks/
    my-deck-1.json
    .lock
  images/
    <URL hash>.png
```

`TEAMY_MTG_CACHE_DIR` can put images elsewhere. `cache show` reports that location;
`cache clean` deletes only recognized cached image files. For a complete offline
backup, copy the database, deck JSON files, and image directory together.

## JSON decks

Each deck is an ordinary, editable UTF-8 file:

```json
{
  "version": 1,
  "name": "my-deck-1",
  "cards": [
    {
      "id": "c43609fb-3cee-44e0-98d0-3ecaba1d5767",
      "name": "Zulaport Cutthroat",
      "quantity": 1
    }
  ]
}
```

Other programs can read these files directly. File names match deck names; names
are case-insensitive and must be valid filenames. The printing ID is authoritative;
the name is a readable label. A copied deck requires those printing IDs in the local
catalog for image/PDF operations. Listing its entries needs no card lookup.

```powershell
teamy-mtg deck card add --deck my-deck-1 "Forest" --quantity 10
teamy-mtg deck card remove --deck my-deck-1 "Forest" --quantity 2
teamy-mtg deck card remove --deck my-deck-1 "Forest"
```

Adding accumulates quantities. Removing without `--quantity` removes the whole
entry. Use a printing ID if multiple entries share a name. Exact full names are
required when adding; for a double-faced card, use `Front // Back` or its ID.
Commands do not enforce deck-format legality or singleton limits. Quantities are
limited to 1–10,000 per printing. CLI edits use file locking and atomic replacement;
external editors should avoid saving concurrently with a CLI edit. Invalid files
and unknown schema versions are reported instead of overwritten.

## Offline search syntax

Search is implemented locally; no search expression is sent to Scryfall.
The parser supports these Scryfall-style features:

| Feature | Example |
|---|---|
| Name substring / exact full name | `zulaport`, `!"Zulaport Cutthroat"` |
| Quoted phrases | `o:"draw a card"` |
| Implicit AND, explicit AND/OR, parentheses | `(t:elf OR t:goblin) mv<=3` |
| Negation | `-t:land`, `NOT (c:u OR c:b)` |
| Name, Oracle text, type, artist | `n:elf`, `o:draw`, `t:creature`, `a:"John Avon"` |
| Regular expressions | `n:/^elvish/`, `o:/draw .* cards/` |
| Colors and color identity | `c:g`, `id:simic`, `id<=gu`, `id=gu`, `id>g`, `id:2` |
| Mana value and numeric stats | `mv<=3`, `cmc=2`, `pow>2`, `tou=4`, `loy>=3` |
| Mana-cost text | `m:{G}{G}` |
| Rarity / set / collector number | `r>=uncommon`, `s:neo`, `cn=42` |
| Format status | `legal:commander`, `f:modern`, `banned:legacy`, `restricted:vintage` |
| Keywords, language, game | `kw:flying`, `lang:en`, `game:paper` |
| Release date | `date>=2024-01-01`, `year:2024` |
| Card properties | `is:commander`, `is:legendary`, `is:dfc`, `is:reserved` |

AND binds more tightly than OR. Text searches are case-insensitive; regular
expressions use Rust regex syntax. `id:g` includes colorless cards and cards whose
identity fits green; `id=g` requires exactly green. `c:g` includes cards containing
green. Color names, WUBRG letters, guild/shard/wedge names, `c`/`colorless`,
`m`/`multicolor`, and numeric color counts are supported.

`is:` / `not:` properties: `commander`, `legendary`, `creature`, `land`,
`permanent`, `spell`, `dfc`, `transform`, `modal_dfc`/`mdfc`, `split`, `adventure`,
`token`, `reserved`, `digital`, `fullart`, `textless`, `reprint`, `promo`.
Commander eligibility considers front faces, Backgrounds, explicit rules text,
Grist, and legendary Vehicles/Spacecraft with power/toughness. It excludes banned
commanders, playtest cards, meld backs, and non-playable digital forms. Add
`legal:commander` to require that format's legality (otherwise eligible digital,
silver-bordered, and acorn cards can appear).

This is a documented subset, **not complete Scryfall parity**. Unsupported fields
and properties return errors. Tags, prices, Scryfall-only rankings, historical
printing aggregation, regex backreferences/lookaround, and dynamic expressions
such as `pow>tou` are not implemented. `o:` and `fo:` both inspect the full stored
Oracle text, including reminder text; mana searches currently match literal cost
text. Legality reflects the last downloaded snapshot.

Results default to 100, sorted by name. Use `--limit` (up to 100,000) and `--offset`
with `card list` or `card search` to page through the full local catalog.

## Images and proxy sheets

`card image list --deck NAME` emits a mapping of card face plus printing ID to an
absolute image path or the string `missing`. Including printing IDs avoids collisions
when a deck contains more than one printing. Damaged cached files count as missing.

`card image sync --deck NAME` downloads and validates missing images, repairs damaged
entries, and reuses existing files. Re-running sync after caching requires no network.
Image URLs are hashed into safe filenames. Requests are spaced and include an
identifying User-Agent. `TEAMY_MTG_API_URL` / `--api-url` can select a download mirror.

```powershell
teamy-mtg proxy-pdf generate --deck my-deck-1 a.pdf --paper letter --offline
teamy-mtg proxy-pdf generate --deck my-deck-1 fronts.pdf --faces front
teamy-mtg proxy-pdf generate --deck my-deck-1 backs.pdf --faces back
teamy-mtg proxy-pdf generate --deck my-deck-1 spaced.pdf --gap 2
```

PDFs contain 63 × 88 mm images in a centered 3 × 3 grid with no gap by default,
so adjacent cards share a single cut edge. Use `--gap MM` for optional spacing
(for example, `--gap 2` restores a 2 mm gap). Gaps must be finite, non-negative,
and small enough to fit the grid and cut marks on the selected paper.
Cut marks sit outside the grid, keeping them off the card artwork.
A4 is the default; Letter is supported. Print at **100% / actual size**, with
printer scaling disabled. Images are embedded as high-quality JPEG data, reused
within the PDF for repeated cards. No Python or external PDF tool is needed at runtime.

`--faces all` (default) prints each face as a separate cutout. These are not
duplex-aligned sheets. Single-image split/adventure cards remain one cutout;
`--faces back` omits single-faced cards. Quantities apply to each selected face.
Empty decks, missing images in offline mode, and existing outputs are errors.
Use `--force` to replace an output. Output is staged and published only after
generation succeeds. PDFs are limited to 10,000 faces.

## Output and development

The template defaults to readable text on a terminal and JSON when stdout is
redirected. `--json` or `--output-format json` makes JSON explicit. `--output-format
text` forces text; CSV is available for flat tabular results such as `deck list`.
Progress/logging goes to stderr. Use `--debug`, `--log-filter`, `--log-file`,
`--stop-after-duration`, and `--completions` as provided by the template.

```powershell
teamy-mtg deck card list --deck my-deck-1 --json > deck-cards.json
cargo +nightly fmt --check
cargo clippy --release --all-targets -- -D warnings
cargo test --release
```

`check-all.ps1` runs the same quality checks. Tests use isolated temporary data and
local HTTP fixtures, including a server-shutdown test that proves cache reuse.
The template's CLI argument roundtrip/fuzz and version-metadata tests are retained.
Windows is validated locally; CI runs Windows and Linux.

References considered: [teamy-rust-cli](https://github.com/TeamDman/teamy-rust-cli),
[mtg-proxies](https://github.com/DiddiZ/mtg-proxies),
[Cockatrice](https://github.com/Cockatrice/Cockatrice),
[MPC Autofill](https://github.com/chilli-axe/mpc-autofill), and
[MTG Familiar](https://github.com/AEFeinstein/mtg-familiar). Card data and scans come
from Scryfall. This tool has no affiliation with Scryfall or Wizards of the Coast.
