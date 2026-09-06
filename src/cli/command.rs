use super::cache::CacheArgs;
use super::card;
use super::context;
use super::db;
use super::deck;
use super::global_args::GlobalArgs;
use super::home::HomeArgs;
use super::output::CliOutput;
use super::proxy_pdf;
use arbitrary::Arbitrary;
use facet::Facet;
use teamy_cancellation::CancellationToken;

/// Offline Magic card search, JSON decks, and printable proxies.
///
#[derive(Facet, Arbitrary, Debug, PartialEq)]
#[repr(u8)]
pub enum Command {
    /// Cache-related commands.
    Cache(CacheArgs),
    /// Home-related commands.
    Home(HomeArgs),
    /// Download or import the local card database.
    Db(db::DbArgs),
    /// Search cards and cache images offline.
    Card(card::CardArgs),
    /// Manage portable JSON decklists.
    Deck(deck::DeckArgs),
    /// Generate printable proxy sheets.
    ProxyPdf(proxy_pdf::ProxyPdfArgs),
}

impl Command {
    /// # Errors
    ///
    /// This function will return an error if the subcommand fails.
    pub async fn invoke(
        self,
        global_args: &GlobalArgs,
        cancellation_token: CancellationToken,
    ) -> eyre::Result<CliOutput> {
        cancellation_token.bail_if_cancelled()?;
        match self {
            Command::Cache(args) => {
                args.invoke(&crate::paths::AppPaths::resolve(
                    global_args.data_dir.as_deref(),
                )?)
                .await
            }
            Command::Home(args) => {
                args.invoke(&crate::paths::AppPaths::resolve(
                    global_args.data_dir.as_deref(),
                )?)
                .await
            }
            Command::Db(args) => args.invoke(&mut context::AppContext::new(
                global_args,
                cancellation_token,
            )?),
            Command::Card(args) => args.invoke(&mut context::AppContext::new(
                global_args,
                cancellation_token,
            )?),
            Command::Deck(args) => args.invoke(&mut context::AppContext::new(
                global_args,
                cancellation_token,
            )?),
            Command::ProxyPdf(args) => args.invoke(&mut context::AppContext::new(
                global_args,
                cancellation_token,
            )?),
        }
    }
}
