use crate::cli::output::CliOutput;
use arbitrary::Arbitrary;
use eyre::Result;
use facet::Facet;

/// Delete the cache files.
#[derive(Facet, Arbitrary, Debug, PartialEq)]
pub struct CacheCleanArgs;

impl CacheCleanArgs {
    /// # Errors
    ///
    /// This function does not return any errors.
    #[expect(
        clippy::unused_async,
        reason = "command invoke methods share the async CLI dispatch shape"
    )]
    pub async fn invoke(self, paths: &crate::paths::AppPaths) -> Result<CliOutput> {
        let value = crate::paths::clean_cache(&paths.image_dir)?;
        let elapsed_duration_warn_threshold =
            crate::elapsed_duration::work_budget(3, value.entries_removed, 20);
        Ok(CliOutput::facet(value, elapsed_duration_warn_threshold))
    }
}
