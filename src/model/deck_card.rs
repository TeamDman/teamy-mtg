use super::Card;
use facet::Facet;
#[derive(Debug, Facet)]
pub struct DeckCard {
    pub quantity: u32,
    pub card: Card,
}
