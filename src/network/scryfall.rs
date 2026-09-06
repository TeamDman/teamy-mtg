use super::bulk::Bulk;
use eyre::Context;
use eyre::ContextCompat;
use eyre::Result;
use eyre::ensure;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::time::Duration;
use std::time::Instant;

#[derive(Debug)]
pub struct Scryfall {
    agent: ureq::Agent,
    base: String,
    last_request: Option<Instant>,
    cancellation: Option<teamy_cancellation::CancellationToken>,
}
impl Scryfall {
    pub fn new(base: &str) -> Result<Self> {
        ensure!(
            base.starts_with("https://") || base.starts_with("http://"),
            "API URL must use HTTP(S)"
        );
        let config = ureq::Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(600)))
            .timeout_connect(Some(Duration::from_secs(20)))
            .user_agent(concat!(
                "teamy-mtg/",
                env!("CARGO_PKG_VERSION"),
                " (https://github.com/TeamDman/teamy-mtg)"
            ))
            .build();
        Ok(Self {
            agent: config.into(),
            base: base.trim_end_matches('/').into(),
            last_request: None,
            cancellation: None,
        })
    }
    pub fn set_cancellation(&mut self, token: teamy_cancellation::CancellationToken) {
        self.cancellation = Some(token);
    }
    pub fn check_cancelled(&self) -> Result<()> {
        if let Some(token) = &self.cancellation {
            token.bail_if_cancelled()?;
        }
        Ok(())
    }
    fn get(&mut self, url: &str) -> Result<ureq::http::Response<ureq::Body>> {
        self.check_cancelled()?;
        ensure!(
            url.starts_with("https://") || url.starts_with("http://"),
            "Download URL must use HTTP(S)"
        );
        if let Some(last) = self.last_request {
            std::thread::sleep(Duration::from_millis(110).saturating_sub(last.elapsed()));
        }
        self.last_request = Some(Instant::now());
        self.agent
            .get(url)
            .header("Accept", "application/json, image/*;q=0.9, */*;q=0.8")
            .call()
            .with_context(|| format!("Cannot download {url}"))
    }
    pub fn download_bulk(&mut self, dir: &Path) -> Result<(tempfile::NamedTempFile, String)> {
        let text = self
            .get(&format!("{}/bulk-data/oracle-cards", self.base))?
            .body_mut()
            .read_to_string()?;
        let metadata: Bulk =
            facet_json::from_str(&text).context("Invalid Scryfall bulk metadata")?;
        let url = metadata
            .jsonl_download_uri
            .or(metadata.download_uri)
            .context("Bulk metadata has no download URI")?;
        tracing::info!(updated_at = %metadata.updated_at, "Downloading Scryfall Oracle Cards");
        let mut file = tempfile::NamedTempFile::new_in(dir)?;
        std::io::copy(&mut self.get(&url)?.body_mut().as_reader(), &mut file)?;
        file.flush()?;
        Ok((
            file,
            format!("Scryfall Oracle Cards {}", metadata.updated_at),
        ))
    }
    pub fn download_image(&mut self, url: &str, output: &mut impl Write) -> Result<()> {
        let mut response = self.get(url)?;
        let mut limited = response.body_mut().as_reader().take(32 * 1024 * 1024 + 1);
        let size = std::io::copy(&mut limited, output)?;
        ensure!(size <= 32 * 1024 * 1024, "Card image exceeds 32 MiB");
        Ok(())
    }
}
