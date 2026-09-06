use crate::model::DeckCard;
use crate::network::Scryfall;
use eyre::Context;
use eyre::Result;
use eyre::ensure;
use sha2::Digest;
use sha2::Sha256;
use std::collections::BTreeMap;
use std::fs;
use std::io::BufReader;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use teamy_cancellation::CancellationToken;
pub fn cache_path(url: &str, cache: &Path) -> PathBuf {
    let extension = if url.split('?').next().is_some_and(|p| p.ends_with(".png")) {
        "png"
    } else if url
        .split('?')
        .next()
        .is_some_and(|p| p.ends_with(".jpg") || p.ends_with(".jpeg"))
    {
        "jpg"
    } else {
        "img"
    };
    cache.join(format!("{:x}.{extension}", Sha256::digest(url.as_bytes())))
}
pub fn load_image(
    url: &str,
    cache: &Path,
    offline: bool,
    api: &mut Scryfall,
) -> Result<image::RgbImage> {
    fs::create_dir_all(cache)?;
    let path = cache_path(url, cache);
    let decode = |path: &Path| -> Result<image::RgbImage> {
        let mut reader =
            image::ImageReader::new(BufReader::new(fs::File::open(path)?)).with_guessed_format()?;
        let mut limits = image::Limits::default();
        limits.max_image_width = Some(8192);
        limits.max_image_height = Some(8192);
        limits.max_alloc = Some(128 * 1024 * 1024);
        reader.limits(limits);
        Ok(reader.decode()?.to_rgb8())
    };
    if path.exists() {
        match decode(&path) {
            Ok(image) => return Ok(image),
            Err(error) if offline => {
                return Err(error).context("Cached card image is damaged; retry without --offline");
            }
            Err(_) => {} // A fresh validated download repairs a damaged entry.
        }
    }
    ensure!(
        !offline,
        "Image not cached for {url}; generate once without --offline"
    );
    let mut staged = tempfile::NamedTempFile::new_in(cache)?;
    api.download_image(url, &mut staged)?;
    staged.flush()?;
    let image = decode(staged.path()).with_context(|| format!("Invalid card image from {url}"))?;
    staged.persist(&path).context("Cannot save image cache")?;
    Ok(image)
}

pub fn list(cards: &[DeckCard], cache: &Path) -> Result<BTreeMap<String, String>> {
    let mut result = BTreeMap::new();
    for item in cards {
        for (name, url) in item.card.images() {
            let path = url.map(|url| cache_path(url, cache));
            let value = match path.filter(|p| p.is_file()) {
                Some(path) if valid_image(&path) => {
                    std::path::absolute(path)?.display().to_string()
                }
                _ => "missing".into(),
            };
            result.insert(format!("{name} [{}]", item.card.id), value);
        }
    }
    Ok(result)
}
fn valid_image(path: &Path) -> bool {
    let Ok(reader) = image::ImageReader::open(path) else {
        return false;
    };
    let Ok(mut reader) = reader.with_guessed_format() else {
        return false;
    };
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(8192);
    limits.max_image_height = Some(8192);
    limits.max_alloc = Some(128 * 1024 * 1024);
    reader.limits(limits);
    reader.decode().is_ok()
}
pub fn sync(
    cards: &[DeckCard],
    cache: &Path,
    api: &mut Scryfall,
    cancellation: &CancellationToken,
) -> Result<BTreeMap<String, String>> {
    let mut seen = std::collections::HashSet::new();
    for item in cards {
        for (name, url) in item.card.images() {
            cancellation.bail_if_cancelled()?;
            let url = url.ok_or_else(|| eyre::eyre!("No image URL for '{name}'"))?;
            if seen.insert(url) {
                tracing::info!("Caching {name}");
                load_image(url, cache, false, api)?;
            }
        }
    }
    list(cards, cache)
}
