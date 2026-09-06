use teamy_mtg::model::Card;
use teamy_mtg::model::Face;
use teamy_mtg::search::Query;
fn green() -> Card {
    Card {
        id: "green".into(),
        name: "Elvish Captain".into(),
        cmc: 3.0,
        mana_cost: "{1}{G}{G}".into(),
        colors: vec!["G".into()],
        color_identity: vec!["G".into()],
        type_line: "Legendary Creature — Elf".into(),
        oracle_text: "Other Elves you control get +1/+1.\nDraw a card.".into(),
        rarity: "rare".into(),
        set: "abc".into(),
        legalities: [("commander".into(), "legal".into())].into(),
        power: Some("2".into()),
        toughness: Some("3".into()),
        ..Card::default()
    }
}
fn matches(query: &str, card: &Card) -> bool {
    Query::parse(query).unwrap().matches(card)
}
#[test]
fn required_query_color_sets_and_comparisons() {
    let card = green();
    for q in [
        "id:g is:commander",
        "id<=simic",
        "id=green",
        "c:g",
        "color>=g",
        "id:1",
        "mv=3",
        "cmc>=2",
        "pow<3",
        "tou=3",
        "r>=uncommon",
        "set:abc",
        "legal:commander",
        "t:elf",
        "o:\"draw a card\"",
        "n:/^elvish/",
        "m:{G}{G}",
    ] {
        assert!(matches(q, &card), "{q}");
    }
    for q in [
        "id=u",
        "id>g",
        "c:colorless",
        "mv>3",
        "r:mythic",
        "banned:commander",
        "-t:elf",
        "name=Captain",
    ] {
        assert!(!matches(q, &card), "{q}");
    }
    let mut colorless = card.clone();
    colorless.colors.clear();
    colorless.color_identity.clear();
    assert!(matches("id:g", &colorless));
    assert!(!matches("c:g", &colorless));
    let mut simic = card.clone();
    simic.color_identity.push("U".into());
    assert!(!matches("id:g", &simic));
    assert!(matches("id>g", &simic));
    assert!(matches("id:m", &simic));
}
#[test]
fn boolean_precedence_negation_quotes_and_regex() {
    let card = green();
    for q in [
        "t:elf OR t:goblin AND c:r",
        "(t:goblin OR t:elf) c:g",
        "NOT (t:goblin OR c:u)",
        "!\"Elvish Captain\"",
        "name!=\"Captain\"",
        "o:/\\S+ Elves/",
    ] {
        assert!(matches(q, &card), "{q}");
    }
    assert!(!matches("(t:elf OR t:goblin) c:r", &card));
    assert!(!matches("!\"Elvish\"", &card));
    for invalid in [
        "",
        "(",
        "()",
        "t:",
        "wat:thing",
        "is:nonexistent",
        "t:elf OR",
        "t:elf )",
        "id:purple",
        "mv:NaN",
        "o:\"unclosed",
        "n:/[/",
        "mv!3",
        "t>elf",
    ] {
        assert!(Query::parse(invalid).is_err(), "{invalid}");
    }
}
#[test]
fn commander_front_face_and_exceptions() {
    let mut card = green();
    card.legalities.insert("commander".into(), "banned".into());
    assert!(!matches("is:commander", &card));
    assert!(!matches("is:commander legal:commander", &card));
    card.legalities.insert("commander".into(), "legal".into());
    card.type_line = "Creature — Human // Legendary Creature — God".into();
    card.card_faces = vec![
        Face {
            name: "Front".into(),
            type_line: "Creature — Human".into(),
            ..Face::default()
        },
        Face {
            name: "Back".into(),
            type_line: "Legendary Creature — God".into(),
            ..Face::default()
        },
    ];
    assert!(!matches("is:commander", &card));
    card.card_faces.clear();
    card.name = "Grist, the Hunger Tide".into();
    card.type_line = "Legendary Planeswalker — Grist".into();
    assert!(matches("is:commander", &card));
    card.name = "Test Ship".into();
    card.type_line = "Legendary Artifact — Spacecraft".into();
    assert!(matches("is:commander", &card));
    card.power = None;
    assert!(!matches("is:commander", &card));
}

#[test]
fn backgrounds_meld_playtests_and_special_sets() {
    let mut card = green();
    card.type_line = "Legendary Enchantment — Background".into();
    assert!(matches("is:commander", &card));
    card.type_line = "Legendary Creature — Elf".into();
    card.all_parts = vec![teamy_mtg::model::RelatedCard {
        id: card.id.clone(),
        component: "meld_result".into(),
    }];
    assert!(!matches("is:commander", &card));
    card.all_parts.clear();
    card.promo_types.push("playtest".into());
    assert!(!matches("is:commander", &card));
    card.promo_types.clear();
    card.legalities.clear();
    card.set_type = "expansion".into();
    assert!(matches("is:commander", &card), "preview cards");
    card.set_type = "funny".into();
    assert!(!matches("is:commander", &card), "special gift cards");
    card.security_stamp = Some("acorn".into());
    assert!(matches("is:commander", &card));
}

#[test]
fn quoted_names_preserve_colons_and_boolean_words() {
    let mut card = green();
    card.name = "Who: What?".into();
    assert!(matches("\"Who: What?\"", &card));
    card.name = "OR".into();
    assert!(matches("\"OR\"", &card));
}
