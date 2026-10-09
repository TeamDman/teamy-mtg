use super::cache_show_report::CacheShowReport;
use crate::cli::output::CliOutput;
use arbitrary::Arbitrary;
use eyre::Result;
use facet::Facet;

/// Show the cache path.
#[derive(Facet, Arbitrary, Debug, PartialEq)]
pub struct CacheShowArgs;

impl CacheShowArgs {
    /// # Errors
    ///
    /// This function does not return any errors.
    #[expect(
        clippy::unused_async,
        reason = "command invoke methods share the async CLI dispatch shape"
    )]
    pub async fn invoke(self, paths: &crate::paths::AppPaths) -> Result<CliOutput> {
        let value = CacheShowReport {
            path: paths.image_dir.display().to_string(),
        };
        let elapsed_duration_warn_threshold = Some(std::time::Duration::from_secs(1));
        Ok(CliOutput::facet(value, elapsed_duration_warn_threshold))
    }
}
