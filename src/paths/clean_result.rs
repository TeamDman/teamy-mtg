/// Result of a cache clean operation.
#[derive(Debug, Default, facet::Facet)]
pub struct CleanResult {
    /// Number of cache entries removed.
    pub entries_removed: usize,
}
