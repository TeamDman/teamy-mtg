use facet::Facet;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Facet, Default)]
pub struct Face {
    pub name: String,
    #[facet(default)]
    pub mana_cost: String,
    #[facet(default)]
    pub type_line: String,
    #[facet(default)]
    pub oracle_text: String,
    #[facet(default)]
    pub colors: Vec<String>,
    #[facet(default)]
    pub image_uris: BTreeMap<String, String>,
}
