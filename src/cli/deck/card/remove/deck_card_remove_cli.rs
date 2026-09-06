use crate::cli::context::AppContext;
use crate::cli::output::CliOutput;
use arbitrary::Arbitrary;
use facet::Facet;
use figue as args;
/// Remove copies; omit quantity to remove the whole entry.
#[derive(Facet, Arbitrary, Debug, PartialEq)]
#[facet(rename_all = "kebab-case")]
pub struct DeckCardRemoveArgs {
    /// Deck name.
    #[facet(args::named)]
    pub deck: String,
    /// Full name or printing ID in this deck.
    #[facet(args::positional)]
    pub card: String,
    /// Copies to remove.
    #[facet(args::named)]
    pub quantity: Option<u32>,
}
impl DeckCardRemoveArgs {
    pub fn invoke(self, ctx: &mut AppContext) -> eyre::Result<CliOutput> {
        ctx.cancellation.bail_if_cancelled()?;
        Ok(CliOutput::facet(ctx.decks.remove(
            &self.deck,
            &self.card,
            self.quantity,
        )?))
    }
}
