use crate::cli::context::AppContext;
use crate::cli::output::CliOutput;
use arbitrary::Arbitrary;
use facet::Facet;
use figue as args;
/// Add copies of a card to a JSON deck.
#[derive(Facet, Arbitrary, Debug, PartialEq)]
#[facet(rename_all = "kebab-case")]
pub struct DeckCardAddArgs {
    /// Deck name.
    #[facet(args::named)]
    pub deck: String,
    /// Exact full card name or printing ID.
    #[facet(args::positional)]
    pub card: String,
    /// Copies to add (default 1).
    #[facet(args::named)]
    pub quantity: Option<u32>,
}
impl DeckCardAddArgs {
    pub fn invoke(self, ctx: &mut AppContext) -> eyre::Result<CliOutput> {
        ctx.cancellation.bail_if_cancelled()?;
        let card = ctx.db.resolve_card(&self.card)?;
        Ok(CliOutput::facet(ctx.decks.add(
            &self.deck,
            &card,
            self.quantity.unwrap_or(1),
        )?))
    }
}
