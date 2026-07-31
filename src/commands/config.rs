use crate::config;
use anyhow::{Context, Result, bail};
use std::path::Path;
use std::process::Command;

pub fn create_config(force: bool) -> Result<()> {
    let path = config::create_default(force)?;

    println!("Config file created at {}.", path.display());

    Ok(())
}

pub fn open_config() -> Result<()> {
    let path = config::ensure_default()?;

    if let Some(editor) = std::env::var_os("EDITOR") {
        return run_editor(&editor.to_string_lossy(), &path);
    }

    for editor in ["vim", "nano"] {
        if run_editor(editor, &path).is_ok() {
            return Ok(());
        }
    }

    bail!("Failed to open config file. Set $EDITOR or install vim or nano.");
}

fn run_editor(editor: &str, path: &Path) -> Result<()> {
    let status = Command::new(editor)
        .arg(path)
        .status()
        .with_context(|| format!("Failed to run editor: {editor}"))?;

    if status.success() {
        Ok(())
    } else {
        bail!("Editor exited with an error: {editor}");
    }
}
