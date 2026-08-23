use crate::commands::mounted::list_mounted_volumes;
use crate::commands::volume::open_file_manager;
use anyhow::{Result, bail};
use dialoguer::{Select, theme::ColorfulTheme};

pub fn open() -> Result<()> {
  let volumes = list_mounted_volumes()?
    .into_iter()
    .filter(|volume| volume.mount_point.is_some())
    .collect::<Vec<_>>();

  if volumes.is_empty() {
    eprintln!("Warning: No mounted volumes found.");
    return Ok(());
  }

  let selected = if volumes.len() == 1 {
    0
  } else {
    let items = volumes
      .iter()
      .map(|volume| volume.label())
      .collect::<Vec<_>>();

    Select::with_theme(&ColorfulTheme::default())
      .with_prompt("Select a mounted volume to open")
      .items(&items)
      .default(0)
      .interact()?
  };

  match &volumes[selected].mount_point {
    Some(mount_point) => open_file_manager(mount_point),
    None => bail!("Volume has no mount point."),
  }
}
