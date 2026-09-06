use facet::Facet;
#[derive(Debug, Facet)]
pub struct DeckSummary {
    pub name: String,
    pub total_cards: u64,
    pub unique_cards: usize,
    pub path: String,
}
