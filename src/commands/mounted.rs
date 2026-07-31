use anyhow::{Context, Result, bail};
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, PartialEq, Eq)]
pub struct MountedVolume {
  pub slot: usize,
  pub volume_path: String,
  pub mount_point: PathBuf,
}

impl MountedVolume {
  pub fn label(&self) -> String {
    format!(
      "Slot {} - {} -> {}",
      self.slot,
      self.volume_path,
      self.mount_point.display()
    )
  }
}

pub fn list_mounted_volumes() -> Result<Vec<MountedVolume>> {
  let output = Command::new("sudo")
    .arg("veracrypt")
    .args(["--text", "--list"])
    .output()
    .context("Failed to run veracrypt. Is it installed?")?;

  let stdout = String::from_utf8_lossy(&output.stdout);
  let stderr = String::from_utf8_lossy(&output.stderr);

  if !output.status.success() {
    if stderr.contains("No volumes mounted") || stdout.contains("No volumes mounted") {
      return Ok(Vec::new());
    }

    let message = stderr.trim();
    if message.is_empty() {
      bail!("Failed to list mounted volumes.");
    }

    bail!("{message}");
  }

  Ok(parse_mounted_volumes(&stdout))
}

fn parse_mounted_volumes(output: &str) -> Vec<MountedVolume> {
  output
    .lines()
    .filter_map(|line| {
      let (slot, description) = line.trim().split_once(':')?;
      let slot = slot.trim().parse().ok()?;
      let description = description.trim();
      let mount_point = description.split_whitespace().last()?;
      let mount_point_start = description.rfind(mount_point)?;
      let volume_path = description[..mount_point_start].trim();

      Some(MountedVolume {
        slot,
        volume_path: volume_path.to_string(),
        mount_point: PathBuf::from(mount_point.trim()),
      })
    })
    .collect()
}

#[cfg(test)]
mod tests {
  use super::{MountedVolume, parse_mounted_volumes};
  use std::path::PathBuf;

  #[test]
  fn parses_mounted_slots_with_mount_points() {
    let output = "/dev mapper line\n1: /home/user/vault.hc   /media/veracrypt1\n2:\t/dev/sdb1\t/media/veracrypt2\n";

    assert_eq!(
      parse_mounted_volumes(output),
      vec![
        MountedVolume {
          slot: 1,
          volume_path: "/home/user/vault.hc".to_string(),
          mount_point: PathBuf::from("/media/veracrypt1"),
        },
        MountedVolume {
          slot: 2,
          volume_path: "/dev/sdb1".to_string(),
          mount_point: PathBuf::from("/media/veracrypt2"),
        },
      ]
    );
  }
}
