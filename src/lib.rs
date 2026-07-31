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
  }

  Ok(())
}
