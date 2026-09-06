use facet::Facet;
#[derive(Facet)]
pub(super) struct Bulk {
    #[facet(default)]
    pub download_uri: Option<String>,
    #[facet(default)]
    pub jsonl_download_uri: Option<String>,
    pub updated_at: String,
}
