use facet::Facet;
#[derive(Facet, Debug)]
pub(super) struct UpdateReport {
    pub cards_imported: usize,
    pub source: String,
    pub data_dir: String,
}
