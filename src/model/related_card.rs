use facet::Facet;

#[derive(Debug, Clone, Facet)]
pub struct RelatedCard {
    pub id: String,
    pub component: String,
}
