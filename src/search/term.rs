use super::helpers::*;
use crate::model::Card;
use eyre::Result;
use eyre::bail;
use eyre::ensure;
use regex::Regex;
#[derive(Debug)]
pub(super) struct Term {
    field: String,
    op: String,
    value: String,
    regex: Option<Regex>,
}
impl Term {
    pub(super) fn parse(word: &str) -> Result<Self> {
        let (field, op, value) = if let Some(value) = word.strip_prefix('!') {
            ("name", "=", value)
        } else if let Some(i) = word.find([':', '=', '<', '>', '!']) {
            let rest = &word[i..];
            let n = if rest.starts_with(">=") || rest.starts_with("<=") || rest.starts_with("!=") {
                2
            } else {
                1
            };
            (&word[..i], &rest[..n], &rest[n..])
        } else {
            ("name", ":", word)
        };
        ensure!(!value.is_empty(), "Missing value in '{word}'");
        let field = match field.to_lowercase().as_str() {
            "n" | "name" => "name",
            "o" | "oracle" | "fo" => "oracle",
            "t" | "type" => "type",
            "c" | "color" | "colour" => "color",
            "id" | "identity" | "ci" => "identity",
            "mv" | "cmc" | "manavalue" => "mv",
            "m" | "mana" => "mana",
            "pow" | "power" => "power",
            "tou" | "toughness" => "toughness",
            "loy" | "loyalty" => "loyalty",
            "r" | "rarity" => "rarity",
            "s" | "set" | "e" | "edition" => "set",
            "f" | "format" | "legal" => "legal",
            "banned" => "banned",
            "restricted" => "restricted",
            "is" => "is",
            "not" => "not",
            "kw" | "keyword" => "keyword",
            "a" | "artist" => "artist",
            "cn" | "number" => "number",
            "date" | "year" => "date",
            "lang" | "language" => "lang",
            "game" => "game",
            other => bail!("Unsupported search field '{other}'. See README.md for offline syntax."),
        }
        .to_string();
        let regex = if value.starts_with('/') && value.ends_with('/') && value.len() > 2 {
            ensure!(
                ["name", "oracle", "type", "artist"].contains(&field.as_str()),
                "Regex not supported for {field}"
            );
            Some(
                regex::RegexBuilder::new(&value[1..value.len() - 1])
                    .case_insensitive(true)
                    .size_limit(1024 * 1024)
                    .build()?,
            )
        } else {
            None
        };
        let value = value.to_lowercase();
        ensure!(
            [":", "=", "!=", "<", ">", "<=", ">="].contains(&op),
            "Unsupported comparison operator"
        );
        if ["mv", "power", "toughness", "loyalty"].contains(&field.as_str()) {
            ensure!(
                value.parse::<f64>().is_ok_and(f64::is_finite),
                "Expected a finite number for {field}"
            );
        }
        if (field == "identity" || field == "color")
            && value != "m"
            && value != "multicolor"
            && value.parse::<u8>().is_err()
        {
            colors(&value)?;
        }
        if field == "is" || field == "not" {
            ensure!(
                [
                    "commander",
                    "legendary",
                    "creature",
                    "land",
                    "permanent",
                    "spell",
                    "dfc",
                    "transform",
                    "modal_dfc",
                    "mdfc",
                    "split",
                    "adventure",
                    "token",
                    "reserved",
                    "digital",
                    "fullart",
                    "textless",
                    "reprint",
                    "promo"
                ]
                .contains(&value.as_str()),
                "Unsupported is: predicate '{value}'"
            );
        }
        if field == "rarity" {
            ensure!(rarity(&value).is_some(), "Unknown rarity '{value}'");
        }
        if [
            "name",
            "oracle",
            "type",
            "mana",
            "set",
            "legal",
            "banned",
            "restricted",
            "is",
            "not",
            "keyword",
            "artist",
            "lang",
            "game",
        ]
        .contains(&field.as_str())
        {
            ensure!(
                [":", "=", "!="].contains(&op),
                "Operator '{op}' is not supported for {field}"
            );
        }
        Ok(Self {
            field,
            op: op.into(),
            value,
            regex,
        })
    }
    pub(super) fn matches(&self, c: &Card) -> bool {
        let value = self.value.as_str();
        let text = |s: &str| {
            if let Some(regex) = &self.regex {
                return regex.is_match(s);
            }
            if self.op == "=" || self.op == "!=" {
                s.to_lowercase() == value
            } else {
                s.to_lowercase().contains(value)
            }
        };
        let result = match self.field.as_str() {
            "name" => text(&c.name),
            "oracle" => text(&c.rules_text()),
            "type" => text(&c.type_line),
            "artist" => text(&c.artist),
            "mana" => text(&c.mana_cost),
            "set" => c.set.eq_ignore_ascii_case(value),
            "lang" => c.lang.eq_ignore_ascii_case(value),
            "game" => c.games.iter().any(|g| g == value),
            "keyword" => c.keywords.iter().any(|k| k.eq_ignore_ascii_case(value)),
            "legal" | "banned" | "restricted" => {
                let format = match value {
                    "edh" => "commander",
                    other => other,
                };
                c.legalities.get(format).is_some_and(|v| {
                    v == &self.field || (self.field == "legal" && v == "restricted")
                })
            }
            "mv" => return compare(c.cmc, &self.op, value.parse().unwrap()),
            "power" | "toughness" | "loyalty" => {
                let stat = match self.field.as_str() {
                    "power" => &c.power,
                    "toughness" => &c.toughness,
                    _ => &c.loyalty,
                };
                return stat
                    .as_ref()
                    .and_then(|s| s.parse::<f64>().ok())
                    .is_some_and(|n| compare(n, &self.op, value.parse().unwrap()));
            }
            "color" | "identity" => {
                let source = if self.field == "color" {
                    &c.colors
                } else {
                    &c.color_identity
                };
                let mask = colors(&source.join("").to_lowercase()).unwrap_or(0);
                if value == "m" || value == "multicolor" {
                    mask.count_ones() > 1
                } else if let Ok(count) = value.parse::<u8>() {
                    return compare(mask.count_ones() as f64, &self.op, f64::from(count));
                } else {
                    let wanted = colors(value).unwrap();
                    return match self.op.as_str() {
                        ":" if self.field == "identity" => mask & wanted == mask,
                        ":" => {
                            if wanted == 0 {
                                mask == 0
                            } else {
                                mask & wanted == wanted
                            }
                        }
                        "=" => mask == wanted,
                        "!=" => mask != wanted,
                        "<=" => mask & wanted == mask,
                        "<" => mask & wanted == mask && mask != wanted,
                        ">=" => mask & wanted == wanted,
                        ">" => mask & wanted == wanted && mask != wanted,
                        _ => false,
                    };
                }
            }
            "rarity" => {
                return rarity(&c.rarity)
                    .is_some_and(|n| compare(n, &self.op, rarity(value).unwrap()));
            }
            "number" => {
                if let (Ok(a), Ok(b)) = (c.collector_number.parse::<f64>(), value.parse::<f64>()) {
                    return compare(a, &self.op, b);
                }
                text(&c.collector_number)
            }
            "date" => {
                let actual = &c.released_at;
                return match self.op.as_str() {
                    ":" => actual.starts_with(value),
                    "=" => actual == value,
                    "!=" => actual != value,
                    ">" => actual.as_str() > value,
                    ">=" => actual.as_str() >= value,
                    "<" => actual.as_str() < value,
                    "<=" => actual.as_str() <= value,
                    _ => false,
                };
            }
            "is" | "not" => {
                let matched = is(c, value);
                if self.field == "not" {
                    !matched
                } else {
                    matched
                }
            }
            _ => false,
        };
        if self.op == "!=" { !result } else { result }
    }
}
