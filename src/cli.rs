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

  /// Open a mounted VeraCrypt volume.
  #[command(
    about = "Open a mounted VeraCrypt volume",
    long_about = "Open a mounted VeraCrypt volume in the configured file manager. If more than one volume is mounted, a selection prompt is shown.",
    after_help = "Examples:\n  vc-cli open\n  vc open"
  )]
  Open,

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

  /// Repair volumes with filesystem errors.
  #[command(
    about = "Repair tools for volumes with filesystem errors",
    long_about = "Open a volume without mounting its filesystem, repair the NTFS filesystem with ntfsfix, mount it temporarily under /tmp, and dismount it when done.",
    after_help = "Examples:\n  vc-cli fix mount ~/vault.hc\n  vc fix list\n  vc fix ntfs\n  vc fix umount"
  )]
  Fix {
    #[command(subcommand)]
    command: FixCommands,
  },
}

#[derive(Subcommand)]
pub enum FixCommands {
  /// Open a volume without mounting its filesystem.
  #[command(
    about = "Open a volume without mounting its filesystem",
    long_about = "Open a VeraCrypt volume with --filesystem=none so the filesystem can be repaired. The password is asked first and sent to VeraCrypt over stdin, never exposed on the command line. If no path is provided, a file picker is opened.",
    after_help = "Examples:\n  vc-cli fix mount ~/vault.hc\n  vc fix mount -k ~/key.bin\n  vc fix mount"
  )]
  Mount {
    /// Path to the VeraCrypt volume file.
    path: Option<PathBuf>,
    /// Path to the keyfile.
    #[arg(short, long)]
    keyfile: Option<String>,
  },

  /// List volumes opened without filesystem.
  #[command(
    about = "List volumes opened without filesystem",
    long_about = "List VeraCrypt volumes currently opened with --filesystem=none, showing the mapped device under /dev/mapper.",
    after_help = "Examples:\n  vc-cli fix list\n  vc fix list"
  )]
  List,

  /// Repair the NTFS filesystem of an opened volume.
  #[command(
    about = "Repair the NTFS filesystem of an opened volume",
    long_about = "Run ntfsfix on a volume opened without filesystem to clear bad sectors and the dirty flag, then mount it under /tmp using ntfs3. If no device is provided, one is selected interactively from the opened volumes.",
    after_help = "Examples:\n  vc-cli fix ntfs\n  vc fix ntfs /dev/mapper/veracrypt1\n  vc fix ntfs 1 -m /tmp/repair\n  vc fix ntfs --no-mount"
  )]
  Ntfs {
    /// Mapped device, slot number, or device name (e.g. veracrypt1).
    mapper: Option<String>,
    /// Where to mount after repairing. Defaults to /tmp/<device>.
    #[arg(short, long)]
    mount_point: Option<PathBuf>,
    /// Do not mount the volume after repairing.
    #[arg(long)]
    no_mount: bool,
  },

  /// Dismount all volumes opened without filesystem.
  #[command(
    about = "Dismount all volumes opened without filesystem",
    long_about = "Unmount the temporary filesystems and dismount every VeraCrypt volume opened with --filesystem=none.",
    after_help = "Examples:\n  vc-cli fix umount\n  vc fix umount"
  )]
  Umount,
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
