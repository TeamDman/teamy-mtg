use std::io::Cursor;
use std::io::Write;
use teamy_mtg::db::Database;
use teamy_mtg::decks::DeckStore;
use teamy_mtg::model::Card;
use teamy_mtg::model::Face;

fn card(id: &str, oracle: &str, name: &str) -> String {
    facet_json::to_string(&Card {
        id: id.into(),
        oracle_id: oracle.into(),
        name: name.into(),
        ..Card::default()
    })
    .unwrap()
}
#[test]
fn imports_are_atomic_and_preserve_json_deck_printings() {
    let dir = tempfile::tempdir().unwrap();
    let mut db = Database::open(dir.path()).unwrap();
    let decks = DeckStore::new(dir.path()).unwrap();
    let old = card("old", "oracle", "Zulaport Cutthroat");
    assert_eq!(
        db.import_reader(Cursor::new(format!("[{old}]")), "fixture")
            .unwrap(),
        1
    );
    decks.create("my-deck-1").unwrap();
    decks
        .add(
            "MY-DECK-1",
            &db.resolve_card("zulaport cutthroat").unwrap(),
            2,
        )
        .unwrap();
    decks
        .add("my-deck-1", &db.resolve_card("old").unwrap(), 1)
        .unwrap();
    let _ = decks.create("MY-DECK-1").unwrap_err();
    let new = card("new", "oracle", "Zulaport Cutthroat");
    for broken in [
        format!("[{new},{{}}]"),
        format!("[{new}] trailing"),
        format!("{new}\nnope"),
        format!("[{new},]"),
        format!("[{new}"),
        "[]".into(),
        "".into(),
    ] {
        let _ = db.import_reader(Cursor::new(broken), "broken").unwrap_err();
        assert_eq!(db.list_cards(None, 100, 0).unwrap()[0].id, "old");
    }
    db.import_reader(Cursor::new(format!("\n {new}\n")), "jsonl")
        .unwrap();
    assert_eq!(db.list_cards(None, 100, 0).unwrap()[0].id, "new");
    let deck = decks.resolve("my-deck-1", &db).unwrap();
    assert_eq!(deck[0].card.id, "old");
    assert_eq!(deck[0].quantity, 3);
    assert_eq!(db.resolve_card("Zulaport Cutthroat").unwrap().id, "new");
    decks
        .remove("my-deck-1", "Zulaport Cutthroat", Some(2))
        .unwrap();
    assert_eq!(decks.list().unwrap()[0].total_cards, 1);
    let _ = decks.remove("my-deck-1", "old", Some(2)).unwrap_err();
    let _ = decks.remove("my-deck-1", "old", Some(0)).unwrap_err();
    drop(db);
    drop(decks);
    let decks = DeckStore::new(dir.path()).unwrap();
    assert_eq!(decks.load("my-deck-1").unwrap().cards[0].quantity, 1);
    let text = std::fs::read_to_string(dir.path().join("decks/my-deck-1.json")).unwrap();
    assert!(text.contains("\"quantity\": 1"));
    decks.remove("my-deck-1", "old", None).unwrap();
    assert!(decks.load("my-deck-1").unwrap().cards.is_empty());
}
#[test]
fn compressed_data_names_and_quantities() {
    let dir = tempfile::tempdir().unwrap();
    let mut db = Database::open(dir.path()).unwrap();
    let decks = DeckStore::new(dir.path()).unwrap();
    let path = dir.path().join("cards.jsonl.gz");
    let mut gz = flate2::write::GzEncoder::new(
        std::fs::File::create(&path).unwrap(),
        flate2::Compression::default(),
    );
    writeln!(gz, "{}", card("one", "one", "Éowyn, Shieldmaiden")).unwrap();
    writeln!(gz, "{}", card("two", "two", "100%_Real")).unwrap();
    gz.finish().unwrap();
    db.import_file(&path, "gzip").unwrap();
    assert_eq!(db.list_cards(Some("éOWYN"), 100, 0).unwrap().len(), 1);
    assert_eq!(db.list_cards(Some("%_"), 100, 0).unwrap().len(), 1);
    assert_eq!(db.list_cards(None, 1, 1).unwrap().len(), 1);
    assert_eq!(db.list_cards(None, 1, 2).unwrap().len(), 0);
    for name in ["", " ", " deck", "deck\n", "../escape", "CON", "a/b"] {
        let _ = decks.create(name).unwrap_err();
    }
    decks.create("my deck").unwrap();
    let card = db.resolve_card("one").unwrap();
    decks.add("my deck", &card, 10000).unwrap();
    let _ = decks.add("my deck", &card, 1).unwrap_err();
    let _ = decks.add("missing", &card, 1).unwrap_err();
    let _ = decks.add("my deck", &card, 0).unwrap_err();
    let _ = db.resolve_card("Shield").unwrap_err();
}
#[test]
fn corrupt_or_future_deck_is_not_overwritten() {
    let dir = tempfile::tempdir().unwrap();
    let decks = DeckStore::new(dir.path()).unwrap();
    decks.create("test").unwrap();
    let path = dir.path().join("decks/test.json");
    for text in [
        r#"{"version":2,"name":"test","cards":[]}"#,
        r#"{"version":1,"name":"test","cards":[],"typo":1}"#,
        "not json",
    ] {
        std::fs::write(&path, text).unwrap();
        let _ = decks.remove("test", "card", None).unwrap_err();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), text);
    }
}
#[test]
fn newer_schema_is_not_modified() {
    let dir = tempfile::tempdir().unwrap();
    let conn = rusqlite::Connection::open(dir.path().join("teamy-mtg.sqlite3")).unwrap();
    conn.execute_batch("PRAGMA user_version=42").unwrap();
    let _ = Database::open(dir.path()).unwrap_err();
    assert_eq!(
        conn.query_row("PRAGMA user_version", [], |r| r.get::<_, u32>(0))
            .unwrap(),
        42
    );
}
#[test]
fn combined_images_and_transform_faces_are_distinguished() {
    let split = Card {
        id: "split".into(),
        name: "Fire // Ice".into(),
        image_uris: [("png".into(), "combined".into())].into(),
        card_faces: vec![
            Face {
                name: "Fire".into(),
                ..Face::default()
            },
            Face {
                name: "Ice".into(),
                ..Face::default()
            },
        ],
        ..Card::default()
    };
    assert_eq!(split.images(), vec![("Fire // Ice", Some("combined"))]);
    let transform = Card {
        id: "dfc".into(),
        name: "Day // Night".into(),
        card_faces: vec![
            Face {
                name: "Day".into(),
                image_uris: [("png".into(), "front".into())].into(),
                ..Face::default()
            },
            Face {
                name: "Night".into(),
                image_uris: [("large".into(), "back".into())].into(),
                ..Face::default()
            },
        ],
        ..Card::default()
    };
    assert_eq!(
        transform.images(),
        vec![("Day", Some("front")), ("Night", Some("back"))]
    );
}
