use std::{
    path::PathBuf,
    sync::{Arc, atomic::AtomicBool},
    time::Duration,
};

use clap::{Parser, Subcommand};
use serde::Serialize;
use serde_json::to_string_pretty;
use thiserror::Error;

#[cfg(feature = "conformance-bridge")]
use crate::conformance_bridge;
use crate::{
    dashboard::{self, DashboardProvider, LocalProvider},
    lifecycle::{Lifecycle, LifecycleError, RuntimeStatus, SystemctlRunner},
    newguy::{self, NewGuyError, Selection},
    newguy_wizard::{self, WizardError},
    runtime::{CharacterId, CharacterIdentity, StorageError, Store, Worker, WorkerError},
    save::{self, SaveError},
};

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
    /// Register a browser .pqw save for local-only managed advancement.
    Register {
        save: PathBuf,
        /// Print credential-safe registration details as JSON.
        #[arg(long)]
        json: bool,
    },
    /// List registered managed characters without exposing save documents.
    List {
        /// Print credential-safe character identities as JSON.
        #[arg(long)]
        json: bool,
    },
    /// Display the persisted canonical state for a managed character.
    ManagedInspect {
        id: String,
        /// Print credential-safe persisted state as JSON.
        #[arg(long)]
        json: bool,
    },
    /// Run a managed character in this process until it receives SIGINT or SIGTERM.
    Worker {
        id: String,
        /// Milliseconds between monotonic-clock persistence updates.
        #[arg(long, default_value_t = 1_000)]
        interval_ms: u64,
    },
    /// Start a managed character's systemd user service.
    Start { id: String },
    /// Stop a managed character's systemd user service.
    Stop { id: String },
    /// Show a managed character's user-service and advisory-lock state.
    Status {
        id: String,
        /// Print credential-safe lifecycle state as JSON.
        #[arg(long)]
        json: bool,
    },
    /// Clear a failed user service state and start the managed character again.
    Recover { id: String },
    /// Open an interactive credential-safe dashboard for a managed character.
    Dashboard {
        id: String,
        /// Milliseconds between persisted-state refreshes.
        #[arg(long, default_value_t = 1_000, value_parser = clap::value_parser!(u64).range(100..=60_000))]
        refresh_ms: u64,
    },
    /// Create and register an offline-only character.
    NewGuy {
        /// Character name. Supplying all three creation traits bypasses the wizard.
        #[arg(long)]
        name: Option<String>,
        /// Race from the bundled ruleset.
        #[arg(long)]
        race: Option<String>,
        /// Class from the bundled ruleset.
        #[arg(long = "class")]
        class: Option<String>,
        /// Print credential-safe registration details as JSON.
        #[arg(long)]
        json: bool,
    },
    #[cfg(feature = "conformance-bridge")]
    /// Test-only stdin/stdout adapter for the disposable browser harness.
    #[command(hide = true)]
    ConformanceBridge,
}

#[derive(Debug, Error)]
pub enum CliError {
    #[error(transparent)]
    Save(#[from] SaveError),
    #[error(transparent)]
    Storage(#[from] StorageError),
    #[error(transparent)]
    Worker(#[from] WorkerError),
    #[error(transparent)]
    Lifecycle(#[from] LifecycleError),
    #[error("could not serialize canonical state: {0}")]
    Json(#[from] serde_json::Error),
    #[error("could not install worker shutdown handler: {0}")]
    Signal(#[from] std::io::Error),
    #[error(transparent)]
    Dashboard(#[from] dashboard::DashboardError),
    #[error(transparent)]
    NewGuy(#[from] NewGuyError),
    #[error(transparent)]
    Wizard(#[from] WizardError),
    #[error("new-guy requires --name, --race, and --class together")]
    PartialCreationInputs,
    #[cfg(feature = "conformance-bridge")]
    #[error(transparent)]
    ConformanceBridge(#[from] conformance_bridge::BridgeError),
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
        Command::Register { save, json } => {
            let character = save::import_file(&save)?;
            let registered = Store::open_default()?.register(&character)?;
            if json {
                println!("{}", to_string_pretty(&registered)?);
            } else {
                println!(
                    "Registered managed character: {}\n  Name: {}\n  Race: {}\n  Class: {}\n  Level: {}",
                    registered.id,
                    registered.identity.name,
                    registered.identity.race,
                    registered.identity.class,
                    registered.identity.level
                );
            }
        }
        Command::List { json } => {
            let characters: Vec<_> = Store::open_default()?
                .list()?
                .into_iter()
                .map(|character| ListedCharacter {
                    id: character.id,
                    identity: character.identity,
                })
                .collect();
            if json {
                println!("{}", to_string_pretty(&characters)?);
            } else if characters.is_empty() {
                println!("No managed characters are registered.");
            } else {
                for character in characters {
                    println!(
                        "{}  {} — {} {} (level {})",
                        character.id,
                        character.identity.name,
                        character.identity.race,
                        character.identity.class,
                        character.identity.level
                    );
                }
            }
        }
        Command::ManagedInspect { id, json } => {
            let character = Store::open_default()?.get(&parse_id(&id)?)?;
            if json {
                println!("{}", to_string_pretty(&character)?);
            } else {
                println!(
                    "Managed character: {}\n\n{}",
                    character.id,
                    character.state.summary()
                );
            }
        }
        Command::Worker { id, interval_ms } => {
            let stop = shutdown_flag()?;
            Worker::start(Store::open_default()?, parse_id(&id)?)?
                .run_until(&stop, Duration::from_millis(interval_ms))?;
        }
        Command::Start { id } => {
            let store = Store::open_default()?;
            let id = parse_id(&id)?;
            Lifecycle::new(&store, SystemctlRunner).start(&id)?;
            println!("Started managed character {id}.");
        }
        Command::Stop { id } => {
            let store = Store::open_default()?;
            let id = parse_id(&id)?;
            Lifecycle::new(&store, SystemctlRunner).stop(&id)?;
            println!("Stopped managed character {id}.");
        }
        Command::Status { id, json } => {
            let store = Store::open_default()?;
            let status = Lifecycle::new(&store, SystemctlRunner).status(&parse_id(&id)?)?;
            print_status(status, json)?;
        }
        Command::Recover { id } => {
            let store = Store::open_default()?;
            let id = parse_id(&id)?;
            Lifecycle::new(&store, SystemctlRunner).recover(&id)?;
            println!("Recovered and started managed character {id}.");
        }
        Command::Dashboard { id, refresh_ms } => {
            let id = parse_id(&id)?;
            let provider = LocalProvider::open_default()?;
            provider.refresh(&id)?;
            let stop = shutdown_flag()?;
            dashboard::run(&provider, id, Duration::from_millis(refresh_ms), &stop)?;
        }
        Command::NewGuy {
            name,
            race,
            class,
            json,
        } => {
            let character = match (name, race, class) {
                (None, None, None) => {
                    let stop = shutdown_flag()?;
                    match newguy_wizard::run(&stop)? {
                        Some(character) => character,
                        None => return Ok(()),
                    }
                }
                (Some(name), Some(race), Some(class)) => newguy::generate_local(
                    &Selection { name, race, class },
                    &crate::ruleset::BUNDLED,
                )?,
                _ => return Err(CliError::PartialCreationInputs),
            };
            let registered = Store::open_default()?.register(&character)?;
            print_registration(registered, json)?;
        }
        #[cfg(feature = "conformance-bridge")]
        Command::ConformanceBridge => conformance_bridge::run_stdio()?,
    }

    fn print_registration(
        registered: crate::runtime::ManagedCharacter,
        json: bool,
    ) -> Result<(), CliError> {
        if json {
            println!("{}", to_string_pretty(&registered)?);
        } else {
            println!(
                "Registered managed character: {}\n  Name: {}\n  Race: {}\n  Class: {}\n  Level: {}",
                registered.id,
                registered.identity.name,
                registered.identity.race,
                registered.identity.class,
                registered.identity.level
            );
        }
        Ok(())
    }
    Ok(())
}

#[derive(Serialize)]
struct ListedCharacter {
    id: CharacterId,
    identity: CharacterIdentity,
}

fn parse_id(value: &str) -> Result<CharacterId, CliError> {
    Ok(CharacterId::parse(value)?)
}

fn shutdown_flag() -> Result<Arc<AtomicBool>, CliError> {
    let stop = Arc::new(AtomicBool::new(false));
    for signal in [
        signal_hook::consts::signal::SIGINT,
        signal_hook::consts::signal::SIGTERM,
    ] {
        signal_hook::flag::register(signal, Arc::clone(&stop))?;
    }
    Ok(stop)
}

fn print_status(status: RuntimeStatus, json: bool) -> Result<(), CliError> {
    if json {
        println!("{}", to_string_pretty(&status)?);
    } else {
        println!(
            "Managed character: {}\n  Name: {}\n  Race: {}\n  Class: {}\n  Level: {}\n  Service: {:?}\n  Runtime ownership: {}",
            status.id,
            status.identity.name,
            status.identity.race,
            status.identity.class,
            status.identity.level,
            status.service,
            if status.runtime_owned {
                "owned"
            } else {
                "not owned"
            }
        );
    }
    Ok(())
}
