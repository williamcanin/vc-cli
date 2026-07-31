use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
  name = "vc-cli",
  about = "A simple interface for the VeraCrypt CLI",
  version
)]
pub struct Cli {
  #[command(subcommand)]
  pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
  /// Mount a VeraCrypt volume.
  #[command(
    about = "Mount a VeraCrypt volume",
    long_about = "Mount a VeraCrypt volume using the VeraCrypt CLI. If no path is provided, a file picker is opened.",
    after_help = "Examples:\n  vc-cli mount ~/vault.hc\n  vc mount /path/to/volume.tc\n  vc mount"
  )]
  Mount {
    /// Path to the VeraCrypt volume file.
    path: Option<PathBuf>,
  },

  /// Dismount a VeraCrypt volume.
  #[command(
    about = "Dismount a VeraCrypt volume",
    long_about = "Dismount mounted VeraCrypt volumes by selecting one or more mounted slots.",
    after_help = "Examples:\n  vc-cli umount\n  vc umount"
  )]
  Umount,

  /// List mounted VeraCrypt volumes.
  #[command(
    about = "List mounted VeraCrypt volumes",
    long_about = "List currently mounted VeraCrypt volumes using the VeraCrypt CLI.",
    after_help = "Examples:\n  vc-cli list\n  vc list"
  )]
  List,

  /// Open the configured file manager.
  #[command(
    about = "Open the file manager",
    long_about = "Open the configured file manager at a path. If no path is provided, the current directory is opened.",
    after_help = "Examples:\n  vc-cli open\n  vc open ~/Documents\n  vc open /mnt"
  )]
  Open {
    /// Directory or file path to open.
    path: Option<PathBuf>,
  },

  /// Manage Vera configuration.
  #[command(
    about = "Manage configuration",
    long_about = "Create and manage optional configuration file.",
    after_help = "Examples:\n  vc-cli config create\n  vc config create --force\n  vc config open"
  )]
  Config {
    #[command(subcommand)]
    command: ConfigCommands,
  },
}

#[derive(Subcommand)]
pub enum ConfigCommands {
  /// Create the default configuration file.
  #[command(
    about = "Create the default configuration file",
    long_about = "Create ~/.config/vc-cli/config.toml with Vera's default settings.",
    after_help = "Examples:\n  vc-cli config create\n  vc config create --force"
  )]
  Create {
    /// Overwrite the configuration file if it already exists.
    #[arg(short, long)]
    force: bool,
  },

  /// Open the configuration file in a terminal editor.
  #[command(
    about = "Open the configuration file",
    long_about = "Open ~/.config/vc-cli/config.toml using $EDITOR. If $EDITOR is not set, Vera tries vim and then nano.",
    after_help = "Examples:\n  vc-cli config open\n  vc config open"
  )]
  Open,
}
