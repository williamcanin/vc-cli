use crate::commands::mount::{choose_slot, free_slots, prompt_keyfile};
use crate::commands::mounted::{MountedVolume, list_mounted_volumes};
use crate::commands::volume::{expand_tilde, select_volume};
use anyhow::{Context, Result, bail};
use dialoguer::{Select, theme::ColorfulTheme};
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn mount(path: Option<PathBuf>, keyfile: Option<String>) -> Result<()> {
  let volume = select_volume(path, "Select the VeraCrypt volume")?;

  let free_slots = free_slots()?;
  if free_slots.is_empty() {
    bail!("No free VeraCrypt slots available.");
  }
  let slot_number = choose_slot(&free_slots)?;
  let keyfile = prompt_keyfile(keyfile)?;

  println!(
    "Opening {} without mounting the filesystem...",
    volume.display()
  );
  println!("VeraCrypt will ask for the password.");

  let mut args = vec![
    "--text".to_string(),
    "--filesystem=none".to_string(),
    "--mount".to_string(),
    volume.to_string_lossy().to_string(),
    format!("--slot={slot_number}"),
    "--pim=0".to_string(),
  ];

  if let Some(k) = keyfile.as_deref() {
    args.push(format!("--keyfiles={k}"));
  }

  let status = Command::new("sudo")
    .arg("veracrypt")
    .args(&args)
    .status()
    .context("Failed to run veracrypt. Is it installed?")?;

  if !status.success() {
    bail!("Failed to open the volume.");
  }

  match repair_volumes()?
    .iter()
    .find(|volume| volume.slot == slot_number)
  {
    Some(volume) => println!("Volume opened at {}.", volume.mount_point.display()),
    None => println!("Volume opened. Run 'vc fix list' to see the mapped device."),
  }

  Ok(())
}

pub fn list() -> Result<()> {
  let volumes = repair_volumes()?;
  if volumes.is_empty() {
    println!("No volumes opened without filesystem.");
    return Ok(());
  }

  for volume in volumes {
    println!("{}", volume.label());
  }

  Ok(())
}

pub fn ntfs(mapper: Option<String>, mount_point: Option<PathBuf>, do_mount: bool) -> Result<()> {
  let device = match mapper {
    Some(raw) => resolve_device(&raw)?,
    None => select_repair_device()?,
  };

  println!("Repairing NTFS on {device}...");

  let status = Command::new("sudo")
    .arg("ntfsfix")
    .arg(&device)
    .status()
    .context("Failed to run ntfsfix. Is it installed?")?;

  if !status.success() {
    bail!("Failed to repair NTFS on {device}.");
  }
  println!("NTFS repaired successfully.");

  if !do_mount {
    return Ok(());
  }

  let target = mount_point.unwrap_or_else(|| default_mount_point(&device));
  std::fs::create_dir_all(&target)
    .with_context(|| format!("Failed to create mount point: {}", target.display()))?;

  let status = Command::new("sudo")
    .arg("mount")
    .args(["-t", "ntfs3"])
    .arg(&device)
    .arg(&target)
    .status()
    .context("Failed to run mount.")?;

  if !status.success() {
    bail!("Failed to mount {device} at {}.", target.display());
  }

  println!("Volume mounted at {}.", target.display());
  Ok(())
}

pub fn umount() -> Result<()> {
  let volumes = repair_volumes()?;
  if volumes.is_empty() {
    eprintln!("Warning: No volumes opened without filesystem.");
    return Ok(());
  }

  for volume in volumes {
    println!("Dismounting slot {}...", volume.slot);

    let _ = Command::new("sudo")
      .arg("umount")
      .arg(&volume.mount_point)
      .status();

    let status = Command::new("sudo")
      .arg("veracrypt")
      .args(["--text", "--unmount", &format!("--slot={}", volume.slot)])
      .status()
      .context("Failed to run veracrypt.")?;

    if !status.success() {
      bail!("Failed to dismount slot {}.", volume.slot);
    }

    cleanup_mount_point(&volume.mount_point);
    println!("Slot {} dismounted successfully.", volume.slot);
  }

  Ok(())
}

fn repair_volumes() -> Result<Vec<MountedVolume>> {
  let volumes = list_mounted_volumes()?;
  Ok(filter_repair_volumes(&volumes))
}

fn filter_repair_volumes(volumes: &[MountedVolume]) -> Vec<MountedVolume> {
  volumes
    .iter()
    .filter(|volume| volume.mount_point.starts_with("/dev/mapper"))
    .cloned()
    .collect()
}

fn select_repair_device() -> Result<String> {
  let volumes = repair_volumes()?;
  if volumes.is_empty() {
    bail!("No volumes opened without filesystem. Use 'vc fix mount' first.");
  }

  let items = volumes
    .iter()
    .map(|volume| volume.label())
    .collect::<Vec<_>>();

  let idx = Select::with_theme(&ColorfulTheme::default())
    .with_prompt("Select the mapped device")
    .default(0)
    .items(&items)
    .interact()?;

  Ok(volumes[idx].mount_point.to_string_lossy().to_string())
}

fn resolve_device(raw: &str) -> Result<String> {
  let raw = raw.trim();
  let expanded = expand_tilde(Path::new(raw));
  if expanded.exists() {
    return Ok(expanded.to_string_lossy().to_string());
  }

  if let Ok(slot) = raw.parse::<usize>() {
    let volumes = repair_volumes()?;
    if let Some(volume) = volumes.iter().find(|volume| volume.slot == slot) {
      return Ok(volume.mount_point.to_string_lossy().to_string());
    }

    bail!("No volume opened without filesystem in slot {slot}.");
  }

  if raw.starts_with("veracrypt") {
    return Ok(format!("/dev/mapper/{raw}"));
  }

  bail!("Device not found: {raw}");
}

fn default_mount_point(device: &str) -> PathBuf {
  let name = Path::new(device)
    .file_name()
    .map(|name| name.to_string_lossy().to_string())
    .unwrap_or_else(|| device.trim_start_matches("/dev/mapper/").to_string());

  PathBuf::from("/tmp").join(name)
}

fn cleanup_mount_point(device: &Path) {
  let name = device.file_name().unwrap_or_default();
  let target = PathBuf::from("/tmp").join(name);
  let _ = Command::new("sudo").arg("rmdir").arg(&target).status();
}

#[cfg(test)]
mod tests {
  use super::{default_mount_point, filter_repair_volumes};
  use crate::commands::mounted::MountedVolume;
  use std::path::PathBuf;

  #[test]
  fn keeps_only_mapper_mount_points() {
    let volumes = vec![
      MountedVolume {
        slot: 1,
        volume_path: "/home/user/vault.hc".to_string(),
        mount_point: PathBuf::from("/media/veracrypt1"),
      },
      MountedVolume {
        slot: 2,
        volume_path: "/home/user/broken.hc".to_string(),
        mount_point: PathBuf::from("/dev/mapper/veracrypt2"),
      },
    ];

    assert_eq!(
      filter_repair_volumes(&volumes),
      vec![MountedVolume {
        slot: 2,
        volume_path: "/home/user/broken.hc".to_string(),
        mount_point: PathBuf::from("/dev/mapper/veracrypt2"),
      }]
    );
  }

  #[test]
  fn derives_default_mount_point_from_device_name() {
    assert_eq!(
      default_mount_point("/dev/mapper/veracrypt1"),
      PathBuf::from("/tmp/veracrypt1")
    );
  }
}
