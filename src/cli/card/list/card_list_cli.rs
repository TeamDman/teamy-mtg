use crate::cli::context::AppContext;
use crate::cli::output::CliOutput;
use arbitrary::Arbitrary;
use facet::Facet;
use figue as args;
/// List cards from the local database.
#[derive(Facet, Arbitrary, Debug, PartialEq)]
#[facet(rename_all = "kebab-case")]
pub struct CardListArgs {
    /// Maximum results (default 100).
    #[facet(args::named)]
    pub limit: Option<u32>,
    /// Skip local results (default 0).
    #[facet(args::named)]
    pub offset: Option<u32>,
}
impl CardListArgs {
    pub fn invoke(self, ctx: &mut AppContext) -> eyre::Result<CliOutput> {
        ctx.cancellation.bail_if_cancelled()?;
        let limit = crate::cli::context::limit(self.limit)?;
        Ok(CliOutput::facet(ctx.db.list_cards(
            None,
            limit,
            self.offset.unwrap_or(0),
        )?))
    }
}
