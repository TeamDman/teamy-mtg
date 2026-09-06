use crate::cli::output::CliOutput;
use arbitrary::Arbitrary;
use eyre::Context;
use eyre::Result;
use facet::Facet;

/// Open the cache path in the platform file manager.
#[derive(Facet, Arbitrary, Debug, PartialEq)]
pub struct CacheOpenArgs;

impl CacheOpenArgs {
    /// # Errors
    ///
    /// This function will return an error if the cache directory cannot be created
    /// or the file manager cannot be launched.
    #[expect(
        clippy::unused_async,
        reason = "command invoke methods share the async CLI dispatch shape"
    )]
    pub async fn invoke(self, paths: &crate::paths::AppPaths) -> Result<CliOutput> {
        std::fs::create_dir_all(paths.image_dir.as_path())?;
        open::that_detached(paths.image_dir.as_path()).wrap_err_with(|| {
            format!(
                "Failed to open {} in file manager",
                paths.image_dir.display()
            )
        })?;
        Ok(CliOutput::none())
    }
}
