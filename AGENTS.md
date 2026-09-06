Each subcommand must have its own directory module.
Each subcommand implementation must live in a new `{}_{}_{}_cli.rs` file that `mod.rs` re-exports to ensure fuzzy finders can find the file easily.
Prefer one type per file.
Use Facet/Figue and facet-json; Serde and clap are forbidden in application code.
