use anyhow::{Context, Result, bail};
use std::process::Command;

pub fn list() -> Result<()> {
    let output = Command::new("sudo")
        .arg("veracrypt")
        .args(["--text", "--list"])
        .output()
        .context("Failed to run veracrypt. Is it installed?")?;

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

    if output.status.success() {
        if stdout.is_empty() {
            println!("No mounted volumes found.");
        } else {
            println!("{stdout}");
        }
    } else if stderr.contains("No volumes mounted") {
        eprintln!("Warning: No mounted volumes found.");
    } else if stderr.is_empty() {
        bail!("Failed to list mounted volumes.");
    } else {
        bail!("{stderr}");
    }

    Ok(())
}
