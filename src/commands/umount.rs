use crate::commands::mounted::list_mounted_volumes;
use anyhow::{Context, Result, bail};
use dialoguer::{MultiSelect, theme::ColorfulTheme};
use std::process::Command;

pub fn umount() -> Result<()> {
  let volumes = list_mounted_volumes()?;
  if volumes.is_empty() {
    eprintln!("Warning: No mounted volumes found.");
    return Ok(());
  }

  let items = volumes
    .iter()
    .map(|volume| volume.label())
    .collect::<Vec<_>>();

  let selected = MultiSelect::with_theme(&ColorfulTheme::default())
    .with_prompt("Select volumes to dismount")
    .items(&items)
    .interact()?;

  if selected.is_empty() {
    println!("No volumes selected.");
    return Ok(());
  }

  for index in selected {
    let volume = &volumes[index];
    println!("Dismounting slot {}...", volume.slot);

    let status = Command::new("sudo")
      .arg("veracrypt")
      .args(["--text", "--unmount", &format!("--slot={}", volume.slot)])
      .status()
      .context("Failed to run veracrypt.")?;

    if status.success() {
      println!("Slot {} dismounted successfully.", volume.slot);
    } else {
      bail!("Failed to dismount slot {}.", volume.slot);
    }
  }

  Ok(())
}
