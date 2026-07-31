use crate::config;
use anyhow::{Context, Result, bail};
use dialoguer::{Input, theme::ColorfulTheme};
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn select_volume(path: Option<PathBuf>, title: &str) -> Result<PathBuf> {
    if let Some(p) = path {
        let expanded = expand_tilde(&p);
        if !expanded.exists() {
            bail!("File not found: {}", expanded.display());
        }
        return Ok(expanded);
    }

    if config::load()?.is_some() {
        open_file_manager(&std::env::current_dir()?)?;

        let raw: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Volume path")
            .interact_text()?;

        let selected = expand_tilde(&PathBuf::from(raw));
        if !selected.exists() {
            bail!("File not found: {}", selected.display());
        }

        return Ok(selected);
    }

    rfd::FileDialog::new()
        .set_title(title)
        .pick_file()
        .context("No file selected")
}

pub fn expand_tilde(path: &Path) -> PathBuf {
    if let Ok(stripped) = path.strip_prefix("~") {
        if let Some(home) = std::env::var_os("HOME").map(PathBuf::from) {
            return home.join(stripped);
        }
    }
    path.to_path_buf()
}

pub fn open_file_manager(path: &Path) -> Result<()> {
    let target = expand_tilde(path);
    let command = config::load()?
        .map(|config| config.general.file_manager)
        .unwrap_or_else(|| "xdg-open".to_string());

    if is_terminal_file_manager(&command) {
        let status = Command::new(&command)
            .arg(&target)
            .status()
            .with_context(|| format!("Failed to open file manager: {command}"))?;

        if !status.success() {
            bail!("File manager exited with an error: {command}");
        }
    } else {
        Command::new(&command)
            .arg(&target)
            .spawn()
            .with_context(|| format!("Failed to open file manager: {command}"))?;
    }

    Ok(())
}

fn is_terminal_file_manager(command: &str) -> bool {
    let name = Path::new(command)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(command);

    matches!(name, "superfile" | "spf")
}
