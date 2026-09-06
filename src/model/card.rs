use super::Face;
use super::RelatedCard;
use facet::Facet;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Facet, Default)]
pub struct Card {
    #[facet(default)]
    pub all_parts: Vec<RelatedCard>,
    #[facet(default)]
    pub promo_types: Vec<String>,
    #[facet(default)]
    pub border_color: String,
    #[facet(default)]
    pub set_type: String,
    #[facet(default)]
    pub security_stamp: Option<String>,
    pub id: String,
    #[facet(default)]
    pub oracle_id: String,
    pub name: String,
    #[facet(default)]
    pub mana_cost: String,
    #[facet(default)]
    pub cmc: f64,
    #[facet(default)]
    pub type_line: String,
    #[facet(default)]
    pub oracle_text: String,
    #[facet(default)]
    pub colors: Vec<String>,
    #[facet(default)]
    pub color_identity: Vec<String>,
    #[facet(default)]
    pub keywords: Vec<String>,
    #[facet(default)]
    pub legalities: BTreeMap<String, String>,
    #[facet(default)]
    pub set: String,
    #[facet(default)]
    pub set_name: String,
    #[facet(default)]
    pub collector_number: String,
    #[facet(default)]
    pub rarity: String,
    #[facet(default)]
    pub layout: String,
    #[facet(default)]
    pub lang: String,
    #[facet(default)]
    pub artist: String,
    #[facet(default)]
    pub power: Option<String>,
    #[facet(default)]
    pub toughness: Option<String>,
    #[facet(default)]
    pub loyalty: Option<String>,
    #[facet(default)]
    pub released_at: String,
    #[facet(default)]
    pub games: Vec<String>,
    #[facet(default)]
    pub reserved: bool,
    #[facet(default)]
    pub digital: bool,
    #[facet(default)]
    pub full_art: bool,
    #[facet(default)]
    pub textless: bool,
    #[facet(default)]
    pub reprint: bool,
    #[facet(default)]
    pub promo: bool,
    #[facet(default)]
    pub image_uris: BTreeMap<String, String>,
    #[facet(default)]
    pub card_faces: Vec<Face>,
}

impl Card {
    pub fn validate(&self) -> eyre::Result<()> {
        eyre::ensure!(
            !self.id.trim().is_empty(),
            "Card is missing its printing ID"
        );
        eyre::ensure!(!self.name.trim().is_empty(), "Card is missing its name");
        Ok(())
    }
    pub fn images(&self) -> Vec<(&str, Option<&str>)> {
        fn best(uris: &BTreeMap<String, String>) -> Option<&str> {
            ["png", "large", "normal"]
                .iter()
                .find_map(|k| uris.get(*k).map(String::as_str))
        }
        if let Some(url) = best(&self.image_uris) {
            vec![(&self.name, Some(url))]
        } else if self.card_faces.iter().any(|f| !f.image_uris.is_empty()) {
            self.card_faces
                .iter()
                .map(|f| (f.name.as_str(), best(&f.image_uris)))
                .collect()
        } else {
            vec![(&self.name, None)]
        }
    }
    pub fn rules_text(&self) -> String {
        if self.oracle_text.is_empty() {
            self.card_faces
                .iter()
                .map(|f| f.oracle_text.as_str())
                .collect::<Vec<_>>()
                .join("\n")
        } else {
            self.oracle_text.clone()
        }
    }
}
