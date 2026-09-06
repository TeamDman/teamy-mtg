use crate::cli::context::AppContext;
use crate::cli::output::CliOutput;
use arbitrary::Arbitrary;
use facet::Facet;
use figue as args;
/// Download missing deck card images for offline use.
#[derive(Facet, Arbitrary, Debug, PartialEq)]
#[facet(rename_all = "kebab-case")]
pub struct CardImageSyncArgs {
    /// Deck name.
    #[facet(args::named)]
    pub deck: String,
}
impl CardImageSyncArgs {
    pub fn invoke(self, ctx: &mut AppContext) -> eyre::Result<CliOutput> {
        ctx.cancellation.bail_if_cancelled()?;
        let cards = ctx.decks.resolve(&self.deck, &ctx.db)?;
        let mut api = ctx.api()?;
        Ok(CliOutput::facet(crate::images::sync(
            &cards,
            &ctx.image_dir,
            &mut api,
            &ctx.cancellation,
        )?))
    }
}
