use anyhow::{Context, Result, bail};
use dialoguer::{MultiSelect, theme::ColorfulTheme};
use std::process::Command;

#[derive(Debug, PartialEq, Eq)]
struct MountedVolume {
  slot: usize,
  description: String,
}

pub fn umount() -> Result<()> {
  let output = Command::new("sudo")
    .arg("veracrypt")
    .args(["--text", "--list"])
    .output()
    .context("Failed to run veracrypt. Is it installed?")?;

  let stdout = String::from_utf8_lossy(&output.stdout);
  let stderr = String::from_utf8_lossy(&output.stderr);

  if !output.status.success() {
    if stderr.contains("No volumes mounted") || stdout.contains("No volumes mounted") {
      eprintln!("Warning: No mounted volumes found.");
      return Ok(());
    }

    let message = stderr.trim();
    if message.is_empty() {
      bail!("Failed to list mounted volumes.");
    }

    bail!("{message}");
  }

  let volumes = parse_mounted_volumes(&stdout);
  if volumes.is_empty() {
    println!("No mounted volumes found.");
    return Ok(());
  }

  let items = volumes
    .iter()
    .map(|volume| format!("Slot {} - {}", volume.slot, volume.description))
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

fn parse_mounted_volumes(output: &str) -> Vec<MountedVolume> {
  output
    .lines()
    .filter_map(|line| {
      let (slot, description) = line.trim().split_once(':')?;
      let slot = slot.trim().parse().ok()?;
      let description = description.trim();

      Some(MountedVolume {
        slot,
        description: if description.is_empty() {
          format!("Slot {slot}")
        } else {
          description.to_string()
        },
      })
    })
    .collect()
}

#[cfg(test)]
mod tests {
  use super::{MountedVolume, parse_mounted_volumes};

  #[test]
  fn parses_mounted_slots() {
    let output = "/dev mapper line\n1: /home/user/vault.hc\n2: /dev/sdb1\n";

    assert_eq!(
      parse_mounted_volumes(output),
      vec![
        MountedVolume {
          slot: 1,
          description: "/home/user/vault.hc".to_string(),
        },
        MountedVolume {
          slot: 2,
          description: "/dev/sdb1".to_string(),
        },
      ]
    );
  }
}
