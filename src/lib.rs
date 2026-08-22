mod cli;
mod commands;
mod config;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands};

pub fn run() -> Result<()> {
  let cli = Cli::parse();

  match cli.command {
    Commands::Mount { path } => commands::mount(path)?,
    Commands::Umount => commands::umount()?,
    Commands::List => commands::list()?,
    Commands::Open => commands::open()?,
    Commands::Config { command } => match command {
      cli::ConfigCommands::Create { force } => commands::create_config(force)?,
      cli::ConfigCommands::Open => commands::open_config()?,
    },
    Commands::Fix { command } => match command {
      cli::FixCommands::Mount { path, keyfile } => commands::fix_mount(path, keyfile)?,
      cli::FixCommands::List => commands::fix_list()?,
      cli::FixCommands::Ntfs {
        mapper,
        mount_point,
        no_mount,
      } => commands::fix_ntfs(mapper, mount_point, !no_mount)?,
      cli::FixCommands::Umount => commands::fix_umount()?,
    },
  }

  Ok(())
}
