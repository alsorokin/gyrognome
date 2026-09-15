use std::path::PathBuf;

use clap::{Parser, Subcommand};
use serde_json::to_string_pretty;
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
    Inspect {
        save: PathBuf,
        /// Print the credential-free canonical state as JSON.
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Error)]
pub enum CliError {
    #[error(transparent)]
    Save(#[from] SaveError),
    #[error("could not serialize canonical state: {0}")]
    Json(#[from] serde_json::Error),
}

pub fn run() -> Result<(), CliError> {
    match Args::parse().command {
        Command::Inspect { save, json } => {
            let character = save::import_file(&save)?;
            if json {
                println!("{}", to_string_pretty(&character)?);
            } else {
                println!("{}", character.summary());
            }
        }
    }
    Ok(())
}
