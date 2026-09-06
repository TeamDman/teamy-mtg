use crate::cli::context::AppContext;
use crate::cli::output::CliOutput;
use arbitrary::Arbitrary;
use facet::Facet;
use figue as args;
/// Create an empty, readable JSON deck.
#[derive(Facet, Arbitrary, Debug, PartialEq)]
#[facet(rename_all = "kebab-case")]
pub struct DeckCreateArgs {
    /// New deck name.
    #[facet(args::positional)]
    pub name: String,
}
impl DeckCreateArgs {
    pub fn invoke(self, ctx: &mut AppContext) -> eyre::Result<CliOutput> {
        ctx.cancellation.bail_if_cancelled()?;
        Ok(CliOutput::facet(ctx.decks.create(&self.name)?))
    }
}
