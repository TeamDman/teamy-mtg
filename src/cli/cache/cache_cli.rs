use super::CacheCommand;
use crate::cli::output::CliOutput;
use arbitrary::Arbitrary;
use eyre::Result;
use facet::Facet;
use figue as args;

/// Cache-related commands.
#[derive(Facet, Arbitrary, Debug, PartialEq)]
pub struct CacheArgs {
    /// The cache subcommand to run.
    #[facet(args::subcommand)]
    pub command: CacheCommand,
}

impl CacheArgs {
    /// # Errors
    ///
    /// This function will return an error if the subcommand fails.
    pub async fn invoke(self, paths: &crate::paths::AppPaths) -> Result<CliOutput> {
        match self.command {
            CacheCommand::Clean(args) => args.invoke(paths).await,
            CacheCommand::Open(args) => args.invoke(paths).await,
            CacheCommand::Show(args) => args.invoke(paths).await,
        }
    }
}
