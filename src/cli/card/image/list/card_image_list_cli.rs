use crate::cli::context::AppContext;
use crate::cli::output::CliOutput;
use arbitrary::Arbitrary;
use facet::Facet;
use figue as args;
/// Map each deck card face to an absolute image path or "missing".
#[derive(Facet, Arbitrary, Debug, PartialEq)]
#[facet(rename_all = "kebab-case")]
pub struct CardImageListArgs {
    /// Deck name.
    #[facet(args::named)]
    pub deck: String,
}
impl CardImageListArgs {
    pub fn invoke(self, ctx: &mut AppContext) -> eyre::Result<CliOutput> {
        ctx.cancellation.bail_if_cancelled()?;
        let cards = ctx.decks.resolve(&self.deck, &ctx.db)?;
        Ok(CliOutput::facet(crate::images::list(
            &cards,
            &ctx.image_dir,
        )?))
    }
}
