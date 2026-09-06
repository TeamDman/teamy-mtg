use super::home_show_report::HomeShowReport;
use crate::cli::output::CliOutput;
use arbitrary::Arbitrary;
use eyre::Result;
use facet::Facet;

/// Show the home path.
#[derive(Facet, Arbitrary, Debug, PartialEq)]
pub struct HomeShowArgs;

impl HomeShowArgs {
    /// # Errors
    ///
    /// This function does not return any errors.
    #[expect(
        clippy::unused_async,
        reason = "command invoke methods share the async CLI dispatch shape"
    )]
    pub async fn invoke(self, paths: &crate::paths::AppPaths) -> Result<CliOutput> {
        Ok(CliOutput::facet(HomeShowReport {
            path: paths.data_dir.display().to_string(),
        }))
    }
}
