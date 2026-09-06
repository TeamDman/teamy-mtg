use crate::cli::context::AppContext;
use crate::cli::output::CliOutput;
use arbitrary::Arbitrary;
use facet::Facet;
use figue as args;
/// Search the offline database using Scryfall-style syntax.
#[derive(Facet, Arbitrary, Debug, PartialEq)]
#[facet(rename_all = "kebab-case")]
pub struct CardSearchArgs {
    /// Search expression, e.g. "id:g is:commander".
    #[facet(args::positional)]
    pub query: String,
    /// Maximum results (default 100).
    #[facet(args::named)]
    pub limit: Option<u32>,
    /// Skip local results (default 0).
    #[facet(args::named)]
    pub offset: Option<u32>,
}
impl CardSearchArgs {
    pub fn invoke(self, ctx: &mut AppContext) -> eyre::Result<CliOutput> {
        ctx.cancellation.bail_if_cancelled()?;
        let query = crate::search::Query::parse(&self.query)?;
        let limit = crate::cli::context::limit(self.limit)?;
        Ok(CliOutput::facet(ctx.db.search(
            &query,
            limit,
            self.offset.unwrap_or(0),
        )?))
    }
}
