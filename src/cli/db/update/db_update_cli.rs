use super::update_report::UpdateReport;
use crate::cli::context::AppContext;
use crate::cli::output::CliOutput;
use arbitrary::Arbitrary;
use facet::Facet;
use figue as args;
/// Refresh the complete offline Oracle card catalog, preserving selected printings.
#[derive(Facet, Arbitrary, Debug, PartialEq)]
#[facet(rename_all = "kebab-case")]
pub struct DbUpdateArgs {
    /// Import a Scryfall JSON array or JSONL file, optionally gzip.
    #[facet(args::named)]
    pub from: Option<String>,
}
impl DbUpdateArgs {
    pub fn invoke(self, ctx: &mut AppContext) -> eyre::Result<CliOutput> {
        ctx.cancellation.bail_if_cancelled()?;
        let (count, source) = if let Some(path) = self.from {
            (
                ctx.db.import_file(std::path::Path::new(&path), &path)?,
                path,
            )
        } else {
            let (file, source) = ctx.api()?.download_bulk(&ctx.data_dir)?;
            (ctx.db.import_file(file.path(), &source)?, source)
        };
        Ok(CliOutput::facet(UpdateReport {
            cards_imported: count,
            source,
            data_dir: ctx.data_dir.display().to_string(),
        }))
    }
}
