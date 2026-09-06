use crate::cli::cache::clean::CacheCleanArgs;
use crate::cli::cache::open::CacheOpenArgs;
use crate::cli::cache::show::CacheShowArgs;
use arbitrary::Arbitrary;
use facet::Facet;

/// Cache subcommands.
#[derive(Facet, Arbitrary, Debug, PartialEq)]
#[repr(u8)]
pub enum CacheCommand {
    /// Clean the cache.
    Clean(CacheCleanArgs),
    /// Open the cache path in the file manager.
    Open(CacheOpenArgs),
    /// Show the cache path.
    Show(CacheShowArgs),
}
