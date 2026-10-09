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
        // Two finite HTTP requests each have a 600-second timeout; imports scale by rows.
        let network_seconds = if self.from.is_some() { 3 } else { 1203 };
        let (count, source) = if let Some(path) = self.from {
            (
                ctx.db.import_file(std::path::Path::new(&path), &path)?,
                path,
            )
        } else {
            let (file, source) = ctx.api()?.download_bulk(&ctx.data_dir)?;
            (ctx.db.import_file(file.path(), &source)?, source)
        };
        let value = UpdateReport {
            cards_imported: count,
            source,
            data_dir: ctx.data_dir.display().to_string(),
        };
        let elapsed_duration_warn_threshold =
            crate::elapsed_duration::work_budget(network_seconds, count, 2);
        Ok(CliOutput::facet(value, elapsed_duration_warn_threshold))
    }
}
