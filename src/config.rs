use anyhow::{anyhow, Result};
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize, Default, Clone)]
pub struct SymposiumConfig {
    pub abstracts: Option<String>,
    pub ordering: Option<String>,
    pub output: Option<String>,
    pub locales: Option<String>,
    pub template: Option<String>,
    pub typst_bin: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct ConfigFile {
    pub symposium: Option<SymposiumConfig>,
}

pub fn default_config_path() -> Result<PathBuf> {
    let cwd = std::env::current_dir()?;
    Ok(cwd.join(".cargo").join("config.toml"))
}

pub fn load_symposium_config(path: &Path) -> Result<Option<SymposiumConfig>> {
    if !path.exists() {
        return Ok(None);
    }
    let contents = std::fs::read_to_string(path)
        .map_err(|e| anyhow!("Failed to read config {}: {}", path.to_string_lossy(), e))?;
    let parsed: ConfigFile = toml::from_str(&contents)
        .map_err(|e| anyhow!("Failed to parse config {}: {}", path.to_string_lossy(), e))?;
    Ok(parsed.symposium)
}

pub fn resolve_cwd_path(raw: &str) -> Result<String> {
    let path = Path::new(raw);
    if path.is_absolute() {
        return Ok(raw.to_string());
    }
    let cwd = std::env::current_dir()?;
    Ok(cwd.join(path).to_string_lossy().to_string())
}
