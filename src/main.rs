mod repository;
mod command;
mod testutil;

use std::path::Path;

use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};

use command::Command;

fn main() -> Result<()> {
    let args = Cli::parse();

    let Some(cmd) = args.command else {
        return Err(anyhow!("No command provided"));
    };

    match cmd {
        Commands::Init { path } => {
            let path = Path::new(&path);
            Command::create(path)?;
        }
    };

    Ok(())
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Init {
        #[arg()]
        path: String,
    },
}

