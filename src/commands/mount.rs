use crate::commands::mounted::list_mounted_volumes;
use crate::commands::volume::{expand_tilde, select_volume};
use anyhow::{Context, Result, bail};
use dialoguer::{Input, Password, Select, theme::ColorfulTheme};
use std::path::PathBuf;
use std::process::Command;

const AVAILABLE_SLOTS: &[usize] = &[1, 2, 3, 4, 5];

pub fn mount(path: Option<PathBuf>) -> Result<()> {
  let volume = select_volume(path, "Select the VeraCrypt volume")?;

  let free_slots = free_slots()?;
  if free_slots.is_empty() {
    bail!("No free VeraCrypt slots available.");
  }

  let slot_items = free_slots
    .iter()
    .map(|slot| format!("Slot {slot}"))
    .collect::<Vec<_>>();

  let slot_idx = Select::with_theme(&ColorfulTheme::default())
    .with_prompt("Select the slot")
    .default(0)
    .items(&slot_items)
    .interact()?;
  let slot_number = free_slots[slot_idx];

  let password = Password::with_theme(&ColorfulTheme::default())
    .with_prompt("Password")
    .interact()?;

  let keyfile_options = ["Yes", "No"];
  let keyfile_idx = Select::with_theme(&ColorfulTheme::default())
    .with_prompt("Use keyfile")
    .default(1)
    .items(keyfile_options)
    .interact()?;

  let keyfile: Option<String> = if keyfile_idx == 0 {
    let raw: String = Input::with_theme(&ColorfulTheme::default())
      .with_prompt("Keyfile path")
      .interact_text()?;
    Some(
      expand_tilde(&PathBuf::from(raw))
        .to_string_lossy()
        .to_string(),
    )
  } else {
    None
  };

  println!("Mounting {}...", volume.display());

  let mut args = vec![
    "--text".to_string(),
    "--mount".to_string(),
    volume.to_string_lossy().to_string(),
    format!("--slot={slot_number}"),
    format!("--password={password}"),
    "--pim=0".to_string(),
    "--non-interactive".to_string(),
  ];

  if let Some(k) = keyfile {
    args.push(format!("--keyfiles={k}"));
  }

  let status = Command::new("sudo")
    .arg("veracrypt")
    .args(&args)
    .status()
    .context("Failed to run veracrypt. Is it installed?")?;

  if status.success() {
    println!("Volume mounted successfully.");
  } else {
    bail!("Failed to mount the volume.");
  }

  Ok(())
}

fn free_slots() -> Result<Vec<usize>> {
  let mounted = list_mounted_volumes()?;
  let used_slots = mounted.iter().map(|volume| volume.slot).collect::<Vec<_>>();

  Ok(free_slots_from_used_slots(&used_slots))
}

fn free_slots_from_used_slots(used_slots: &[usize]) -> Vec<usize> {
  AVAILABLE_SLOTS
    .iter()
    .copied()
    .filter(|slot| !used_slots.contains(slot))
    .collect()
}

#[cfg(test)]
mod tests {
  use super::free_slots_from_used_slots;

  #[test]
  fn excludes_used_slots() {
    assert_eq!(free_slots_from_used_slots(&[1, 3]), vec![2, 4, 5]);
  }
}
