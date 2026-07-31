use crate::commands::volume::{expand_tilde, select_volume};
use anyhow::{Context, Result, bail};
use dialoguer::{Input, Password, Select, theme::ColorfulTheme};
use std::path::PathBuf;
use std::process::Command;

pub fn mount(path: Option<PathBuf>) -> Result<()> {
    let volume = select_volume(path, "Select the VeraCrypt volume")?;

    let password = Password::with_theme(&ColorfulTheme::default())
        .with_prompt("Password")
        .interact()?;

    let slots = ["Slot 1", "Slot 2", "Slot 3", "Slot 4", "Slot 5"];
    let slot_idx = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select the slot")
        .default(0)
        .items(slots)
        .interact()?;
    let slot_number = slot_idx + 1;

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
