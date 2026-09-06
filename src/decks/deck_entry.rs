use facet::Facet;

#[derive(Debug, Clone, Facet)]
#[facet(deny_unknown_fields)]
pub struct DeckEntry {
    pub id: String,
    pub name: String,
    pub quantity: u32,
}
