use super::DeckEntry;
use facet::Facet;
#[derive(Debug, Clone, Facet)]
#[facet(deny_unknown_fields)]
pub struct Deck {
    pub version: u32,
    pub name: String,
    pub cards: Vec<DeckEntry>,
}
