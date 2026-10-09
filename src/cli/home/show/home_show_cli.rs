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
        let value = HomeShowReport {
            path: paths.data_dir.display().to_string(),
        };
        let elapsed_duration_warn_threshold = Some(std::time::Duration::from_secs(1));
        Ok(CliOutput::facet(value, elapsed_duration_warn_threshold))
    }
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn path_inspection_has_a_one_second_budget() {
        let paths = crate::paths::AppPaths {
            data_dir: "synthetic-home".into(),
            image_dir: "synthetic-cache".into(),
        };
        let output = super::HomeShowArgs.invoke(&paths).await.unwrap();
        assert_eq!(
            output.elapsed_duration_warn_threshold,
            Some(std::time::Duration::from_secs(1))
        );
    }
}
