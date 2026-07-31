use anyhow::{Context, Result, bail};
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

const DEFAULT_CONFIG: &str = r#"[general]
file_manager = "nautilus"
"#;

#[derive(Debug, Deserialize)]
pub struct Config {
  pub general: GeneralConfig,
}

#[derive(Debug, Deserialize)]
pub struct GeneralConfig {
  pub file_manager: String,
}

pub fn config_path() -> Result<PathBuf> {
  let home = std::env::var_os("HOME")
    .map(PathBuf::from)
    .context("Could not determine the home directory")?;

  Ok(home.join(".config").join("vc-cli").join("config.toml"))
}

pub fn load() -> Result<Option<Config>> {
  let path = config_path()?;

  if !path.exists() {
    return Ok(None);
  }

  let contents = fs::read_to_string(&path)
    .with_context(|| format!("Failed to read config file: {}", path.display()))?;

  let config = toml::from_str(&contents)
    .with_context(|| format!("Failed to parse config file: {}", path.display()))?;

  Ok(Some(config))
}

pub fn create_default(force: bool) -> Result<PathBuf> {
  let path = config_path()?;

  if path.exists() && !force {
    bail!(
      "Config file already exists: {}. Use --force to overwrite it.",
      path.display()
    );
  }

  if let Some(parent) = path.parent() {
    fs::create_dir_all(parent)
      .with_context(|| format!("Failed to create config directory: {}", parent.display()))?;
  }

  fs::write(&path, DEFAULT_CONFIG)
    .with_context(|| format!("Failed to write config file: {}", path.display()))?;

  Ok(path)
}

pub fn ensure_default() -> Result<PathBuf> {
  let path = config_path()?;

  if path.exists() {
    return Ok(path);
  }

  create_default(false)
}
