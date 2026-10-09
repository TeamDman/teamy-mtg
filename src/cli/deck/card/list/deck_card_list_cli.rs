use crate::cli::context::AppContext;
use crate::cli::output::CliOutput;
use arbitrary::Arbitrary;
use facet::Facet;
use figue as args;
/// List the entries stored in a JSON deck.
#[derive(Facet, Arbitrary, Debug, PartialEq)]
#[facet(rename_all = "kebab-case")]
pub struct DeckCardListArgs {
    /// Deck name.
    #[facet(args::named)]
    pub deck: String,
}
impl DeckCardListArgs {
    pub fn invoke(self, ctx: &mut AppContext) -> eyre::Result<CliOutput> {
        ctx.cancellation.bail_if_cancelled()?;
        let value = ctx.decks.load(&self.deck)?.cards;
        let elapsed_duration_warn_threshold =
            crate::elapsed_duration::work_budget(3, value.len(), 20);
        Ok(CliOutput::facet(value, elapsed_duration_warn_threshold))
    }
}
