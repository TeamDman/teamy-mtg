use super::DeckCommand;
use crate::cli::context::AppContext;
use crate::cli::output::CliOutput;
use arbitrary::Arbitrary;
use facet::Facet;
use figue as args;

#[derive(Facet, Arbitrary, Debug, PartialEq)]
pub struct DeckArgs {
    #[facet(args::subcommand)]
    pub command: DeckCommand,
}
impl DeckArgs {
    pub fn invoke(self, ctx: &mut AppContext) -> eyre::Result<CliOutput> {
        match self.command {
            DeckCommand::List(args) => args.invoke(ctx),
            DeckCommand::Create(args) => args.invoke(ctx),
            DeckCommand::Card(args) => args.invoke(ctx),
        }
    }
}
