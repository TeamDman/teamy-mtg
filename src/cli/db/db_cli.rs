use super::DbCommand;
use crate::cli::context::AppContext;
use crate::cli::output::CliOutput;
use arbitrary::Arbitrary;
use facet::Facet;
use figue as args;

#[derive(Facet, Arbitrary, Debug, PartialEq)]
pub struct DbArgs {
    #[facet(args::subcommand)]
    pub command: DbCommand,
}
impl DbArgs {
    pub fn invoke(self, ctx: &mut AppContext) -> eyre::Result<CliOutput> {
        match self.command {
            DbCommand::Update(args) => args.invoke(ctx),
        }
    }
}
