use crate::model::Card;
use eyre::Result;
use eyre::bail;
pub(super) fn compare(a: f64, op: &str, b: f64) -> bool {
    match op {
        ":" | "=" => a == b,
        "!=" => a != b,
        ">" => a > b,
        ">=" => a >= b,
        "<" => a < b,
        "<=" => a <= b,
        _ => false,
    }
}
pub(super) fn colors(value: &str) -> Result<u8> {
    let value = match value {
        "white" => "w",
        "blue" => "u",
        "black" => "b",
        "red" => "r",
        "green" => "g",
        "colorless" | "c" => "",
        "azorius" => "wu",
        "dimir" => "ub",
        "rakdos" => "br",
        "gruul" => "rg",
        "selesnya" => "gw",
        "orzhov" => "wb",
        "izzet" => "ur",
        "golgari" => "bg",
        "boros" => "rw",
        "simic" => "gu",
        "esper" => "wub",
        "grixis" => "ubr",
        "jund" => "brg",
        "naya" => "rgw",
        "bant" => "gwu",
        "abzan" => "wbg",
        "jeskai" => "wur",
        "sultai" => "ubg",
        "mardu" => "wbr",
        "temur" => "urg",
        other => other,
    };
    let mut mask = 0;
    for c in value.chars() {
        let bit = match c {
            'w' => 1,
            'u' => 2,
            'b' => 4,
            'r' => 8,
            'g' => 16,
            _ => bail!("Unknown color '{c}'"),
        };
        mask |= bit;
    }
    Ok(mask)
}
pub(super) fn rarity(value: &str) -> Option<f64> {
    Some(match value {
        "c" | "common" => 0.0,
        "u" | "uncommon" => 1.0,
        "r" | "rare" => 2.0,
        "m" | "mythic" => 3.0,
        "s" | "special" => 4.0,
        "b" | "bonus" => 5.0,
        _ => return None,
    })
}
pub(super) fn is(c: &Card, value: &str) -> bool {
    let ty = c.type_line.to_lowercase();
    match value {
        "commander" => {
            // Front faces only; MTG Familiar CardDbAdapter uses the same eligibility cases.
            let front = c.card_faces.first();
            let ty = front
                .map_or(c.type_line.as_str(), |f| f.type_line.as_str())
                .to_lowercase();
            let text = front
                .map_or(c.oracle_text.as_str(), |f| f.oracle_text.as_str())
                .to_lowercase();
            let back = c
                .all_parts
                .iter()
                .any(|part| part.id == c.id && part.component == "meld_result");
            let eligible = c.legalities.get("commander").is_none_or(|v| v != "banned")
                && !c.promo_types.iter().any(|p| p == "playtest")
                && (!c.digital
                    || c.legalities
                        .values()
                        .any(|v| v == "legal" || v == "restricted"))
                && (c.set_type != "funny"
                    || c.legalities
                        .values()
                        .any(|v| v == "legal" || v == "restricted")
                    || c.border_color == "silver"
                    || c.security_stamp.as_deref() == Some("acorn"));
            eligible
                && !back
                && !c.layout.contains("token")
                && ((ty.contains("legendary") && ty.contains("creature"))
                    || (ty.contains("legendary") && ty.contains("background"))
                    || text.contains("can be your commander")
                    || c.name == "Grist, the Hunger Tide"
                    || (ty.contains("legendary")
                        && (ty.contains("vehicle") || ty.contains("spacecraft"))
                        && c.power.is_some()
                        && c.toughness.is_some()))
        }
        "legendary" | "creature" | "land" => ty.contains(value),
        "permanent" => [
            "artifact",
            "battle",
            "creature",
            "enchantment",
            "land",
            "planeswalker",
        ]
        .iter()
        .any(|t| ty.contains(t)),
        "spell" => !ty.contains("land"),
        "dfc" => c.card_faces.iter().any(|f| !f.image_uris.is_empty()),
        "mdfc" => c.layout == "modal_dfc",
        "transform" | "modal_dfc" | "split" | "adventure" => c.layout == value,
        "token" => c.layout.contains("token"),
        "reserved" => c.reserved,
        "digital" => c.digital,
        "fullart" => c.full_art,
        "textless" => c.textless,
        "reprint" => c.reprint,
        "promo" => c.promo,
        _ => false,
    }
}
