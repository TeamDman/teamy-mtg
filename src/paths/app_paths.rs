use std::path::PathBuf;
#[derive(Debug)]
pub struct AppPaths {
    pub data_dir: PathBuf,
    pub image_dir: PathBuf,
}
impl AppPaths {
    pub fn resolve(data_dir: Option<&str>) -> eyre::Result<Self> {
        let data_dir = match data_dir {
            Some(path) => PathBuf::from(path),
            None => super::AppHome::resolve()?.0,
        };
        let data_dir = std::path::absolute(data_dir)?;
        let image_dir = std::path::absolute(
            std::env::var_os(super::APP_CACHE_ENV_VAR)
                .map(PathBuf::from)
                .unwrap_or_else(|| data_dir.join("images")),
        )?;
        Ok(Self {
            data_dir,
            image_dir,
        })
    }
}
