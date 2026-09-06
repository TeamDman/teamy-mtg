use super::global_args::GlobalArgs;
use crate::db::Database;
use crate::decks::DeckStore;
use crate::network::Scryfall;
use eyre::Result;
use eyre::ensure;
use std::path::PathBuf;
use teamy_cancellation::CancellationToken;

#[derive(Debug)]
pub struct AppContext {
    pub data_dir: PathBuf,
    pub image_dir: PathBuf,
    pub db: Database,
    pub decks: DeckStore,
    pub cancellation: CancellationToken,
    api_url: String,
}
impl AppContext {
    pub fn new(args: &GlobalArgs, cancellation: CancellationToken) -> Result<Self> {
        let crate::paths::AppPaths {
            data_dir,
            image_dir,
        } = crate::paths::AppPaths::resolve(args.data_dir.as_deref())?;
        let mut db = Database::open(&data_dir)?;
        db.set_cancellation(cancellation.clone());
        let decks = DeckStore::new(&data_dir)?;
        Ok(Self {
            db,
            decks,
            data_dir,
            image_dir,
            cancellation,
            api_url: args
                .api_url
                .clone()
                .or_else(|| std::env::var("TEAMY_MTG_API_URL").ok())
                .unwrap_or_else(|| "https://api.scryfall.com".into()),
        })
    }
    pub fn api(&self) -> Result<Scryfall> {
        let mut api = Scryfall::new(&self.api_url)?;
        api.set_cancellation(self.cancellation.clone());
        Ok(api)
    }
}
pub fn limit(limit: Option<u32>) -> Result<u32> {
    let limit = limit.unwrap_or(100);
    ensure!(
        (1..=100000).contains(&limit),
        "Limit must be between 1 and 100000"
    );
    Ok(limit)
}
