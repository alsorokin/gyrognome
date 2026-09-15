use std::path::PathBuf;

use clap::{Parser, Subcommand};
use thiserror::Error;

use crate::save::{self, SaveError};

#[derive(Debug, Parser)]
#[command(
    name = "gyrognome",
    about = "Offline Progress Quest compatibility tools"
)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Display canonical state from a browser .pqw export without contacting a server.
    Inspect { save: PathBuf },
}

#[derive(Debug, Error)]
pub enum CliError {
    #[error(transparent)]
    Save(#[from] SaveError),
}

pub fn run() -> Result<(), CliError> {
    match Args::parse().command {
        Command::Inspect { save } => {
            let character = save::import_file(&save)?;
            println!("{}", character.summary());
        }
    }
    Ok(())
}
