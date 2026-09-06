use super::CardCommand;
use crate::cli::context::AppContext;
use crate::cli::output::CliOutput;
use arbitrary::Arbitrary;
use facet::Facet;
use figue as args;

#[derive(Facet, Arbitrary, Debug, PartialEq)]
pub struct CardArgs {
    #[facet(args::subcommand)]
    pub command: CardCommand,
}
impl CardArgs {
    pub fn invoke(self, ctx: &mut AppContext) -> eyre::Result<CliOutput> {
        match self.command {
            CardCommand::List(args) => args.invoke(ctx),
            CardCommand::Search(args) => args.invoke(ctx),
            CardCommand::Image(args) => args.invoke(ctx),
        }
    }
}
