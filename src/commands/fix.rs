use crate::commands::mount::{choose_slot, free_slots, prompt_keyfile};
use crate::commands::mounted::{MountedVolume, list_mounted_volumes};
use crate::commands::volume::{expand_tilde, select_volume};
use anyhow::{Context, Result, anyhow, bail};
use dialoguer::{Password, Select, theme::ColorfulTheme};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub fn mount(path: Option<PathBuf>, keyfile: Option<String>) -> Result<()> {
  let volume = select_volume(path, "Select the VeraCrypt volume")?;

  let free_slots = free_slots()?;
  if free_slots.is_empty() {
    bail!("No free VeraCrypt slots available.");
  }
  let slot_number = choose_slot(&free_slots)?;

  let password = Password::with_theme(&ColorfulTheme::default())
    .with_prompt("Password")
    .interact()?;

  let keyfile = prompt_keyfile(keyfile)?;

  println!(
    "Opening {} without mounting the filesystem...",
    volume.display()
  );

  let mut args = vec![
    "--text".to_string(),
    "--filesystem=none".to_string(),
    "--mount".to_string(),
    volume.to_string_lossy().to_string(),
    format!("--slot={slot_number}"),
    "--pim=0".to_string(),
    "--stdin".to_string(),
    "--non-interactive".to_string(),
  ];

  if let Some(k) = keyfile.as_deref() {
    args.push(format!("--keyfiles={k}"));
  }

  let mut child = Command::new("sudo")
    .arg("veracrypt")
    .args(&args)
    .stdin(Stdio::piped())
    .spawn()
    .context("Failed to run veracrypt. Is it installed?")?;

  child
    .stdin
    .take()
    .context("Failed to open veracrypt stdin.")?
    .write_all(format!("{password}\n").as_bytes())
    .context("Failed to send password to veracrypt.")?;

  let status = child.wait().context("Failed to wait for veracrypt.")?;

  if !status.success() {
    bail!("Failed to open the volume.");
  }

  match repair_devices()?
    .iter()
    .find(|device| device.slot == slot_number)
  {
    Some(device) => println!("Volume opened at {}.", device.device.display()),
    None => println!("Volume opened. Run 'vc fix list' to see the mapped device."),
  }

  Ok(())
}

pub fn list() -> Result<()> {
  let devices = repair_devices()?;
  if devices.is_empty() {
    println!("No repair-mode volumes found.");
    return Ok(());
  }

  for device in devices {
    println!("{}", device.label());
  }

  Ok(())
}

pub fn ntfs(mapper: Option<String>, mount_point: Option<PathBuf>, do_mount: bool) -> Result<()> {
  let repair = match mapper {
    Some(raw) => resolve_device(&raw)?,
    None => select_repair_device()?,
  };

  println!("Repairing NTFS on {}...", repair.device.display());

  let status = Command::new("sudo")
    .arg("ntfsfix")
    .args(["-b", "-d"])
    .arg(&repair.device)
    .status()
    .context("Failed to run ntfsfix. Is it installed?")?;

  if !status.success() {
    bail!("Failed to repair NTFS on {}.", repair.device.display());
  }
  println!("NTFS repaired successfully.");

  if !do_mount {
    return Ok(());
  }

  let target = mount_point.unwrap_or_else(|| default_mount_point(&repair.device));
  std::fs::create_dir_all(&target)
    .with_context(|| format!("Failed to create mount point: {}", target.display()))?;

  let status = Command::new("sudo")
    .arg("mount")
    .args(["-t", "ntfs3"])
    .arg(&repair.device)
    .arg(&target)
    .status()
    .context("Failed to run mount.")?;

  if !status.success() {
    bail!(
      "Failed to mount {} at {}.",
      repair.device.display(),
      target.display()
    );
  }

  println!("Volume mounted at {}.", target.display());
  Ok(())
}

pub fn umount() -> Result<()> {
  let devices = repair_devices()?;
  if devices.is_empty() {
    eprintln!("Warning: No repair-mode volumes found.");
    return Ok(());
  }

  for repair in devices {
    println!("Dismounting slot {}...", repair.slot);

    match &repair.mount_point {
      Some(mount_point) => {
        let _ = Command::new("sudo").arg("umount").arg(mount_point).status();
      }
      None => {
        let _ = Command::new("sudo")
          .arg("umount")
          .arg(&repair.device)
          .status();
      }
    }

    let status = Command::new("sudo")
      .arg("veracrypt")
      .args(["--text", "--unmount", &format!("--slot={}", repair.slot)])
      .status()
      .context("Failed to run veracrypt.")?;

    if !status.success() {
      bail!("Failed to dismount slot {}.", repair.slot);
    }

    cleanup_mount_point(&repair.device);
    println!("Slot {} dismounted successfully.", repair.slot);
  }

  Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RepairDevice {
  slot: usize,
  volume_path: String,
  device: PathBuf,
  mount_point: Option<PathBuf>,
}

impl RepairDevice {
  fn is_mounted(&self) -> bool {
    self.mount_point.is_some()
  }

  fn label(&self) -> String {
    match &self.mount_point {
      Some(mount_point) => format!(
        "Slot {} - {} -> {} -> {} (mounted)",
        self.slot,
        self.volume_path,
        self.device.display(),
        mount_point.display()
      ),
      None => format!(
        "Slot {} - {} -> {} (no filesystem)",
        self.slot,
        self.volume_path,
        self.device.display()
      ),
    }
  }
}

fn repair_devices() -> Result<Vec<RepairDevice>> {
  let proc_mounts = fs::read_to_string("/proc/mounts").context("Failed to read /proc/mounts.")?;

  let names = fs::read_dir("/dev/mapper")
    .context("Failed to read /dev/mapper.")?
    .filter_map(|entry| entry.ok())
    .map(|entry| entry.file_name().to_string_lossy().to_string())
    .collect::<Vec<_>>();

  let listed = list_mounted_volumes().unwrap_or_default();
  let mut devices = collect_repair_devices(&names, &proc_mounts, &listed);
  devices.sort_by_key(|device| device.slot);
  Ok(devices)
}

fn collect_repair_devices(
  names: &[String],
  proc_mounts: &str,
  listed: &[MountedVolume],
) -> Vec<RepairDevice> {
  let mut devices = names
    .iter()
    .filter_map(|name| {
      let slot = name.strip_prefix("veracrypt")?.parse::<usize>().ok()?;
      let device = Path::new("/dev/mapper").join(name);

      let mount_point = match find_mount_point_in(proc_mounts, &device) {
        Some(mount_point) if mount_point.starts_with("/tmp") => Some(mount_point),
        Some(_) => return None,
        None => None,
      };

      Some(RepairDevice {
        slot,
        volume_path: listed_volume_path(listed, slot),
        device,
        mount_point,
      })
    })
    .collect::<Vec<_>>();
  devices.sort_by_key(|device| device.slot);
  devices
}

fn find_mount_point_in(proc_mounts: &str, device: &Path) -> Option<PathBuf> {
  let target = device.to_string_lossy();

  for line in proc_mounts.lines() {
    let mut fields = line.split_whitespace();
    if fields.next() == Some(target.as_ref()) {
      return fields.next().map(PathBuf::from);
    }
  }

  None
}

fn listed_volume_path(listed: &[MountedVolume], slot: usize) -> String {
  listed
    .iter()
    .find(|volume| volume.slot == slot)
    .map(|volume| volume.volume_path.clone())
    .unwrap_or_default()
}

fn mapper_device_name(slot: usize) -> PathBuf {
  Path::new("/dev/mapper").join(format!("veracrypt{slot}"))
}

fn select_repair_device() -> Result<RepairDevice> {
  let devices = repair_devices()?;
  if devices.is_empty() {
    bail!("No volumes opened without filesystem. Use 'vc fix mount' first.");
  }

  let pending = devices
    .iter()
    .filter(|device| !device.is_mounted())
    .cloned()
    .collect::<Vec<_>>();

  if pending.is_empty() {
    bail!("All volumes are already mounted. Use 'vc fix umount' to dismount them.");
  }

  let items = pending
    .iter()
    .map(|device| device.label())
    .collect::<Vec<_>>();

  let idx = Select::with_theme(&ColorfulTheme::default())
    .with_prompt("Select the mapped device")
    .default(0)
    .items(&items)
    .interact()?;

  Ok(pending[idx].clone())
}

fn resolve_device(raw: &str) -> Result<RepairDevice> {
  let raw = raw.trim();

  if let Ok(slot) = raw.parse::<usize>() {
    return repair_devices()?
      .into_iter()
      .find(|d| d.slot == slot)
      .ok_or_else(|| anyhow!("No volume opened without filesystem in slot {slot}."));
  }

  let expanded = expand_tilde(Path::new(raw));
  let text = expanded.to_string_lossy();

  let name = Path::new(&*text)
    .file_name()
    .and_then(|name| name.to_str())
    .unwrap_or(&text);

  if !name.starts_with("veracrypt") {
    bail!("Device not found: {raw}");
  }

  let slot = name["veracrypt".len()..]
    .parse::<usize>()
    .map_err(|_| anyhow!("Invalid VeraCrypt device: {raw}"))?;

  let device = mapper_device_name(slot);
  if !device.exists() {
    bail!("Device not found: {}", device.display());
  }

  Ok(RepairDevice {
    slot,
    volume_path: String::new(),
    device,
    mount_point: None,
  })
}

fn default_mount_point(device: &Path) -> PathBuf {
  let name = device
    .file_name()
    .map(|name| name.to_string_lossy().to_string())
    .unwrap_or_else(|| {
      device
        .to_string_lossy()
        .trim_start_matches("/dev/mapper/")
        .to_string()
    });

  PathBuf::from("/tmp").join(name)
}

fn cleanup_mount_point(device: &Path) {
  let name = device.file_name().unwrap_or_default();
  let target = PathBuf::from("/tmp").join(name);
  let _ = Command::new("sudo").arg("rmdir").arg(&target).status();
}

#[cfg(test)]
mod tests {
  use super::{RepairDevice, collect_repair_devices, default_mount_point, find_mount_point_in};
  use crate::commands::mounted::MountedVolume;
  use std::path::{Path, PathBuf};

  fn listed(slot: usize, volume_path: &str) -> MountedVolume {
    MountedVolume {
      slot,
      volume_path: volume_path.to_string(),
      mount_point: None,
    }
  }

  #[test]
  fn collects_pending_and_tmp_mounted_devices() {
    let names = vec![
      "control".to_string(),
      "home".to_string(),
      "veracrypt1".to_string(),
      "veracrypt2".to_string(),
      "veracrypt3".to_string(),
    ];
    let proc_mounts = "/dev/mapper/veracrypt2 /media/veracrypt2 ntfs3 rw 0 0\n/dev/mapper/veracrypt3 /tmp/veracrypt3 ntfs3 rw 0 0\n";
    let volumes = vec![listed(1, "/seagate/volumes/volume0")];

    assert_eq!(
      collect_repair_devices(&names, proc_mounts, &volumes),
      vec![
        RepairDevice {
          slot: 1,
          volume_path: "/seagate/volumes/volume0".to_string(),
          device: PathBuf::from("/dev/mapper/veracrypt1"),
          mount_point: None,
        },
        RepairDevice {
          slot: 3,
          volume_path: String::new(),
          device: PathBuf::from("/dev/mapper/veracrypt3"),
          mount_point: Some(PathBuf::from("/tmp/veracrypt3")),
        },
      ]
    );
  }

  #[test]
  fn collects_device_without_listed_volume_info() {
    let names = vec!["veracrypt3".to_string()];

    assert_eq!(
      collect_repair_devices(&names, "", &[]),
      vec![RepairDevice {
        slot: 3,
        volume_path: String::new(),
        device: PathBuf::from("/dev/mapper/veracrypt3"),
        mount_point: None,
      }]
    );
  }

  #[test]
  fn sorts_collected_devices_by_slot() {
    let names = vec!["veracrypt5".to_string(), "veracrypt2".to_string()];
    let devices = collect_repair_devices(&names, "", &[]);
    let slots = devices.iter().map(|device| device.slot).collect::<Vec<_>>();
    assert_eq!(slots, vec![2, 5]);
  }

  #[test]
  fn finds_mount_point_in_proc_mounts() {
    let proc_mounts =
      "sysfs /sys sysfs rw 0 0\n/dev/mapper/veracrypt1 /tmp/veracrypt1 ntfs3 rw 0 0\n";

    assert_eq!(
      find_mount_point_in(proc_mounts, Path::new("/dev/mapper/veracrypt1")),
      Some(PathBuf::from("/tmp/veracrypt1"))
    );
    assert_eq!(
      find_mount_point_in(proc_mounts, Path::new("/dev/mapper/veracrypt2")),
      None
    );
  }

  #[test]
  fn derives_default_mount_point_from_device_name() {
    assert_eq!(
      default_mount_point(Path::new("/dev/mapper/veracrypt1")),
      PathBuf::from("/tmp/veracrypt1")
    );
  }
}
