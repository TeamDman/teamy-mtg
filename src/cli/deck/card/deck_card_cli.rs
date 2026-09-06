use super::DeckCardCommand;
use crate::cli::context::AppContext;
use crate::cli::output::CliOutput;
use arbitrary::Arbitrary;
use facet::Facet;
use figue as args;

#[derive(Facet, Arbitrary, Debug, PartialEq)]
pub struct DeckCardArgs {
    #[facet(args::subcommand)]
    pub command: DeckCardCommand,
}
impl DeckCardArgs {
    pub fn invoke(self, ctx: &mut AppContext) -> eyre::Result<CliOutput> {
        match self.command {
            DeckCardCommand::Add(args) => args.invoke(ctx),
            DeckCardCommand::List(args) => args.invoke(ctx),
            DeckCardCommand::Remove(args) => args.invoke(ctx),
        }
    }
}
