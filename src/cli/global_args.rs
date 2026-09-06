//! Global arguments that apply to all commands.

use crate::cli::output::OutputFormat;
use arbitrary::Arbitrary;
use facet::Facet;
use figue::{self as args};
use teamy_cancellation::StopAfterArgs;

/// Global arguments that apply to all commands.
#[derive(Facet, Arbitrary, Debug, Default, PartialEq)]
#[facet(rename_all = "kebab-case")]
pub struct GlobalArgs {
    /// Database and deck directory. Overrides TEAMY_MTG_DATA_DIR.
    #[facet(args::named)]
    pub data_dir: Option<String>,
    /// Scryfall API origin for downloads (supports local mirrors).
    #[facet(args::named)]
    pub api_url: Option<String>,
    /// Shorthand for --output-format json.
    #[facet(args::named, default)]
    pub json: bool,

    /// Enable debug logging, including backtraces on panics.
    #[facet(args::named, default)]
    pub debug: bool,

    /// Log level filter directive.
    #[facet(args::named)]
    #[arbitrary(default)]
    pub log_filter: Option<String>,

    /// Write structured ndjson logs.
    ///
    /// If a file path is provided, logs are written to that file.
    /// If a directory path is provided, a filename like `log_<timestamp>.ndjson`
    /// is generated in that directory.
    /// If omitted, no JSON log file is written.
    #[facet(args::named)]
    #[arbitrary(default)]
    pub log_file: Option<String>,

    /// Request graceful cancellation from tracing/Tracy spans, log messages, or elapsed time.
    #[facet(flatten, default)]
    #[arbitrary(default)]
    pub stop_after: StopAfterArgs,

    /// Render command output as `text`, `json`, or `csv`.
    ///
    /// If omitted, the generated CLI uses `text` for interactive terminals and
    /// `json` when stdout is redirected.
    #[facet(args::named)]
    #[arbitrary(default)]
    pub output_format: Option<OutputFormat>,
}
