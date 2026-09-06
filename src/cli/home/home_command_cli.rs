use crate::cli::home::open::HomeOpenArgs;
use crate::cli::home::show::HomeShowArgs;
use arbitrary::Arbitrary;
use facet::Facet;

/// Home subcommands.
#[derive(Facet, Arbitrary, Debug, PartialEq)]
#[repr(u8)]
pub enum HomeCommand {
    /// Open the home path in the file manager.
    Open(HomeOpenArgs),
    /// Show the home path.
    Show(HomeShowArgs),
}
