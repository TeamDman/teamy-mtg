use super::CardImageCommand;
use crate::cli::context::AppContext;
use crate::cli::output::CliOutput;
use arbitrary::Arbitrary;
use facet::Facet;
use figue as args;

#[derive(Facet, Arbitrary, Debug, PartialEq)]
pub struct CardImageArgs {
    #[facet(args::subcommand)]
    pub command: CardImageCommand,
}
impl CardImageArgs {
    pub fn invoke(self, ctx: &mut AppContext) -> eyre::Result<CliOutput> {
        match self.command {
            CardImageCommand::Sync(args) => args.invoke(ctx),
            CardImageCommand::List(args) => args.invoke(ctx),
        }
    }
}
