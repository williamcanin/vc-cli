use crate::config;
use anyhow::{Context, Result, bail};
use dialoguer::{Input, theme::ColorfulTheme};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn select_volume(path: Option<PathBuf>, title: &str) -> Result<PathBuf> {
  if let Some(p) = path {
    let expanded = expand_tilde(&p);
    if !expanded.exists() {
      bail!("File not found: {}", expanded.display());
    }
    return Ok(expanded);
  }

  if let Some(config) = config::load()? {
    let command = config.general.file_manager;
    let selected = if is_terminal_file_manager(&command) {
      select_with_terminal_file_manager(&command, &std::env::current_dir()?)?
    } else {
      open_file_manager_command(&command, &std::env::current_dir()?)?;

      let raw: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Volume path")
        .interact_text()?;

      expand_tilde(&PathBuf::from(raw))
    };

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
  if let Ok(stripped) = path.strip_prefix("~")
    && let Some(home) = std::env::var_os("HOME").map(PathBuf::from)
  {
    return home.join(stripped);
  }
  path.to_path_buf()
}

pub fn open_file_manager(path: &Path) -> Result<()> {
  let target = expand_tilde(path);
  let command = config::load()?
    .map(|config| config.general.file_manager)
    .unwrap_or_else(|| "xdg-open".to_string());

  open_file_manager_command(&command, &target)
}

fn open_file_manager_command(command: &str, path: &Path) -> Result<()> {
  let target = expand_tilde(path);

  if is_terminal_file_manager(command) {
    let status = Command::new(command)
      .arg(&target)
      .status()
      .with_context(|| format!("Failed to open file manager: {command}"))?;

    if !status.success() {
      bail!("File manager exited with an error: {command}");
    }
  } else {
    Command::new(command)
      .arg(&target)
      .spawn()
      .with_context(|| format!("Failed to open file manager: {command}"))?;
  }

  Ok(())
}

fn select_with_terminal_file_manager(command: &str, path: &Path) -> Result<PathBuf> {
  let chooser_file = chooser_file_path()?;
  let target = expand_tilde(path);

  let status = Command::new(command)
    .arg("--chooser-file")
    .arg(&chooser_file)
    .arg(&target)
    .status()
    .with_context(|| format!("Failed to open file manager: {command}"))?;

  if !status.success() {
    let _ = fs::remove_file(&chooser_file);
    bail!("File manager exited with an error: {command}");
  }

  let selected = fs::read_to_string(&chooser_file)
    .with_context(|| format!("Failed to read chooser file: {}", chooser_file.display()))?;
  let _ = fs::remove_file(&chooser_file);

  let selected = selected.trim();
  if selected.is_empty() {
    bail!("No file selected");
  }

  Ok(expand_tilde(&PathBuf::from(selected)))
}

fn chooser_file_path() -> Result<PathBuf> {
  let timestamp = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .context("System clock is before the Unix epoch")?
    .as_nanos();

  Ok(std::env::temp_dir().join(format!(
    "vera-chooser-{}-{timestamp}.txt",
    std::process::id()
  )))
}

fn is_terminal_file_manager(command: &str) -> bool {
  let name = Path::new(command)
    .file_name()
    .and_then(|name| name.to_str())
    .unwrap_or(command);

  matches!(name, "superfile" | "spf")
}

#[cfg(test)]
mod tests {
  use super::select_with_terminal_file_manager;
  use std::fs;
  use std::os::unix::fs::PermissionsExt;
  use std::path::Path;

  #[test]
  fn reads_selected_path_from_chooser_file() {
    let temp_dir = std::env::temp_dir().join(format!("vera-volume-test-{}", std::process::id()));
    fs::create_dir_all(&temp_dir).unwrap();

    let selected = temp_dir.join("vault.hc");
    fs::write(&selected, "").unwrap();

    let script = temp_dir.join("fake-spf");
    fs::write(
            &script,
            format!(
                "#!/bin/sh\nwhile [ \"$#\" -gt 0 ]; do\n  if [ \"$1\" = \"--chooser-file\" ]; then\n    shift\n    printf '%s\\n' '{}' > \"$1\"\n    exit 0\n  fi\n  shift\ndone\nexit 1\n",
                selected.display()
            ),
        )
        .unwrap();

    let mut permissions = fs::metadata(&script).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&script, permissions).unwrap();

    let result =
      select_with_terminal_file_manager(script.to_str().unwrap(), Path::new(".")).unwrap();

    assert_eq!(result, selected);

    let _ = fs::remove_dir_all(temp_dir);
  }
}
