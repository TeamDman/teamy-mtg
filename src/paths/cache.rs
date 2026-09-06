use super::CleanResult;
use std::path::Path;
use std::path::PathBuf;
use std::sync::LazyLock;
use tracing::warn;

/// The cache home directory for API responses.
/// This MUST NOT be used within functions outside of a top-level resolution function; any function relying on
/// a [`CacheHome`] must take it as a parameter to ensure testing is straightforward.
pub static CACHE_DIR: LazyLock<CacheHome> = LazyLock::new(|| match CacheHome::resolve() {
    Ok(c) => c,
    Err(e) => {
        warn!("Failed to resolve cache home: {}", e);
        CacheHome(std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
    }
});

/// Helper that resolves the application cache directory.
#[derive(Clone, Debug)]
pub struct CacheHome(pub PathBuf);

impl CacheHome {
    /// Resolve the `CacheHome` according to:
    /// * If [`super::APP_CACHE_ENV_VAR`] env var is set, use that directory
    /// * Otherwise use the platform `ProjectDirs::cache_dir()`
    ///
    /// # Errors
    ///
    /// This function will return an error if the cache directory cannot be determined.
    pub fn resolve() -> eyre::Result<CacheHome> {
        if let Ok(override_dir) = std::env::var(super::APP_CACHE_ENV_VAR) {
            return Ok(CacheHome(PathBuf::from(override_dir)));
        }
        Ok(CacheHome(super::AppHome::resolve()?.0.join("images")))
    }
}

impl std::ops::Deref for CacheHome {
    type Target = Path;

    fn deref(&self) -> &Self::Target {
        self.0.as_path()
    }
}

/// Clean the entire API response cache directory.
///
/// # Errors
///
/// This function will return an error if removing files or directories fails.
pub fn clean_cache(cache_dir: &Path) -> eyre::Result<CleanResult> {
    let mut result = CleanResult::default();
    if !cache_dir.exists() {
        return Ok(result);
    }
    for entry in std::fs::read_dir(cache_dir)? {
        let entry = entry?;
        let path = entry.path();
        let own_name = path.file_stem().is_some_and(|s| {
            s.to_string_lossy().len() == 64
                && s.to_string_lossy().chars().all(|c| c.is_ascii_hexdigit())
        });
        let image = path
            .extension()
            .is_some_and(|e| e == "png" || e == "jpg" || e == "img");
        if entry.file_type()?.is_file() && own_name && image {
            std::fs::remove_file(path)?;
            result.entries_removed += 1;
        }
    }
    Ok(result)
}
