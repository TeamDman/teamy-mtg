use super::HomeCommand;
use crate::cli::output::CliOutput;
use arbitrary::Arbitrary;
use eyre::Result;
use facet::Facet;
use figue as args;

/// Home-related commands.
#[derive(Facet, Arbitrary, Debug, PartialEq)]
pub struct HomeArgs {
    /// The home subcommand to run.
    #[facet(args::subcommand)]
    pub command: HomeCommand,
}

impl HomeArgs {
    /// # Errors
    ///
    /// This function will return an error if the subcommand fails.
    pub async fn invoke(self, paths: &crate::paths::AppPaths) -> Result<CliOutput> {
        match self.command {
            HomeCommand::Open(args) => args.invoke(paths).await,
            HomeCommand::Show(args) => args.invoke(paths).await,
        }
    }
}
