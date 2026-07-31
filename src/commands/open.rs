use crate::commands::volume::{expand_tilde, open_file_manager};
use anyhow::Result;
use std::path::PathBuf;

pub fn open(path: Option<PathBuf>) -> Result<()> {
  let path = path
    .map(|p| expand_tilde(&p))
    .unwrap_or(std::env::current_dir()?);

  open_file_manager(&path)
}
