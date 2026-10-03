use std::{
    io::{self, BufRead, Write},
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
    compatibility::CompatibilityProfile,
    dashboard::{self, DashboardProvider, LocalProvider},
    lifecycle::{Lifecycle, LifecycleError, RuntimeStatus, SystemctlRunner},
    newguy::{self, NewGuyError, Selection},
    newguy_wizard::{self, WizardError},
    reporting::{self, DeliveryOutcome, HttpsTransport, ReportingError},
    runtime::{
        CharacterId, CharacterIdentity, ManagedInspectionState, StorageError, Store, Worker,
        WorkerError,
    },
    save::{self, SaveError},
};

#[derive(Debug, Parser)]
#[command(name = "gyro", about = "Offline Progress Quest compatibility tools")]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Display canonical state from a supported browser or desktop save without contacting a server.
    Inspect {
        save: PathBuf,
        /// Print the credential-free canonical state as JSON.
        #[arg(long)]
        json: bool,
    },
    /// Register a supported browser or desktop save for managed advancement.
    Register {
        save: PathBuf,
        /// Print credential-safe registration details as JSON.
        #[arg(long)]
        json: bool,
    },
    /// List safe identities, persisted compatibility profiles, and online eligibility.
    List {
        /// Print credential-safe character identities as JSON.
        #[arg(long)]
        json: bool,
    },
    /// Display credential-safe persisted state, profile, provenance, and eligibility.
    ManagedInspect {
        id: String,
        /// Print credential-safe persisted state as JSON.
        #[arg(long)]
        json: bool,
    },
    /// Run a managed character using its persisted continuation profile.
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
    /// Delete an inactive managed character after confirmation.
    Delete { id: String },
    /// Submit one confirmed leaderboard report when the persisted profile is eligible.
    Report { id: String },
    /// Save an eligible motto and deliver one report; --clear stores an empty motto.
    Motto {
        id: String,
        #[arg(required_unless_present = "clear", conflicts_with = "clear")]
        text: Option<String>,
        #[arg(long)]
        clear: bool,
    },
    /// Join or change an eligible guild; an explicitly empty designation leaves.
    Guild { id: String, designation: String },
    /// Open an interactive credential-safe dashboard with an immediate manual Brag action.
    Dashboard {
        id: Option<String>,
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
    #[error("could not read deletion confirmation: {0}")]
    Confirmation(#[source] std::io::Error),
    #[error(transparent)]
    Reporting(#[from] ReportingError),
    #[error("managed character {0} is running; stop it before deleting")]
    CharacterRunning(CharacterId),
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
        Command::Inspect { save, json } => match save::import_supported_file(&save)? {
            save::ImportedSave::Browser(character) => {
                if json {
                    println!("{}", to_string_pretty(&character)?);
                } else {
                    println!("{}", character.summary());
                }
            }
            save::ImportedSave::Desktop(desktop) => {
                let inspection = save::inspect_desktop(&desktop);
                if json {
                    println!("{}", to_string_pretty(&inspection)?);
                } else {
                    println!(
                        "Desktop save inspection (target: desktop-6.4.4)\n{}",
                        to_string_pretty(&inspection)?
                    );
                }
            }
        },
        Command::Register { save, json } => {
            let mut store = Store::open_default()?;
            match save::import_supported_file(&save)? {
                save::ImportedSave::Browser(character) => {
                    let registered = store.register(&character)?;
                    print_registration(&registered, json)?;
                }
                save::ImportedSave::Desktop(desktop) => {
                    let mut random = newguy::OsRandom::open()?;
                    let registered = store.register_desktop(&desktop, &mut random)?;
                    print_registration(&registered, json)?;
                }
            }
        }
        Command::List { json } => {
            let characters = Store::open_default()?.list_managed()?;
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
                        character.identity.level,
                    );
                    println!(
                        "    Profile: {}",
                        compatibility_profile_name(character.compatibility.profile)
                    );
                    for eligibility in &character.compatibility.online_eligibility {
                        println!(
                            "    {:?}: {:?}",
                            eligibility.operation, eligibility.decision
                        );
                    }
                    if let Some(notice) = character.compatibility.notice {
                        println!("    {notice}");
                    }
                }
            }
        }
        Command::ManagedInspect { id, json } => {
            let inspection = Store::open_default()?.managed_inspection(&parse_id(&id)?)?;
            if json {
                println!("{}", to_string_pretty(&inspection)?);
            } else {
                match &inspection.state {
                    ManagedInspectionState::Browser(character) => println!(
                        "Managed character: {}\nProfile: browser\n\n{}",
                        character.id,
                        character.state.summary()
                    ),
                    ManagedInspectionState::Desktop644(character) => {
                        println!(
                            "Managed character: {}\nProfile: desktop-6.4.4\n\n{}",
                            character.id,
                            to_string_pretty(&inspection)?
                        );
                    }
                }
                println!(
                    "Rested: {} ms available | {}x",
                    inspection.rested.available_ms, inspection.rested.active_multiplier,
                );
                if let Some(notice) = inspection.rested_online_notice {
                    println!("{notice}");
                }
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
            if let Some(warning) = store.start_warning(&id)? {
                eprintln!("Warning: {warning}");
            }
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
            if let Some(warning) = store.start_warning(&id)? {
                eprintln!("Warning: {warning}");
            }
            Lifecycle::new(&store, SystemctlRunner).recover(&id)?;
            println!("Recovered and started managed character {id}.");
        }
        Command::Delete { id } => {
            let id = parse_id(&id)?;
            let mut store = Store::open_default()?;
            let identity = store.identity(&id)?;
            println!(
                "Delete managed character {id}?\n  Name: {}\n  Race: {}\n  Class: {}\n  Level: {}\nType yes to confirm: ",
                identity.name, identity.race, identity.class, identity.level
            );
            io::stdout().flush().map_err(CliError::Confirmation)?;
            if !delete_confirmed(&mut io::stdin().lock())? {
                println!("Deletion cancelled.");
                return Ok(());
            }
            match store.remove(&id) {
                Ok(()) => {}
                Err(StorageError::AlreadyOwned(_)) => {
                    return Err(CliError::CharacterRunning(id));
                }
                Err(error) => return Err(error.into()),
            }
            println!("Deleted managed character {id}.");
        }
        Command::Report { id } => {
            let id = parse_id(&id)?;
            let store = Store::open_default()?;
            let identity = store.identity(&id)?;
            println!(
                "Submit one leaderboard report for managed character {id}?\n  Name: {}\n  Race: {}\n  Class: {}\n  Level: {}\nType yes to confirm: ",
                identity.name, identity.race, identity.class, identity.level
            );
            io::stdout().flush().map_err(CliError::Confirmation)?;
            if !delete_confirmed(&mut io::stdin().lock())? {
                println!("Report cancelled.");
                return Ok(());
            }
            #[cfg(feature = "enrollment-test-transport")]
            let transport = reporting::TestEnrollmentTransport::from_environment();
            #[cfg(not(feature = "enrollment-test-transport"))]
            let transport = HttpsTransport;
            let result = reporting::submit_manual_brag(&store, &id, &transport)?;
            match result.outcome {
                DeliveryOutcome::Delivered => println!("Leaderboard report delivered."),
                DeliveryOutcome::EndpointRejected => {
                    println!("Leaderboard report was not accepted by the endpoint.")
                }
                DeliveryOutcome::DeliveryFailed => {
                    println!("Leaderboard report could not be delivered.")
                }
            }
        }
        Command::Motto { id, text, clear: _ } => {
            let id = parse_id(&id)?;
            let mut store = Store::open_default()?;
            #[cfg(feature = "enrollment-test-transport")]
            let transport = reporting::TestEnrollmentTransport::from_environment();
            #[cfg(not(feature = "enrollment-test-transport"))]
            let transport = HttpsTransport;
            let result =
                reporting::set_motto(&mut store, &id, text.as_deref().unwrap_or(""), &transport)?;
            println!("{}", result.outcome.motto_message());
        }
        Command::Guild { id, designation } => {
            let id = parse_id(&id)?;
            let mut store = Store::open_default()?;
            #[cfg(feature = "enrollment-test-transport")]
            let result = reporting::set_guild_for_test(
                &mut store,
                &id,
                &designation,
                &reporting::TestEnrollmentTransport::from_environment(),
            )?;
            #[cfg(not(feature = "enrollment-test-transport"))]
            let result = reporting::set_guild(&mut store, &id, &designation, &HttpsTransport)?;
            println!("{}", result.outcome.message());
        }
        Command::Dashboard { id, refresh_ms } => {
            let stop = shutdown_flag()?;
            let id = match id {
                Some(id) => parse_id(&id)?,
                None => {
                    let store = Store::open_default()?;
                    let lifecycle = Lifecycle::new(&store, SystemctlRunner);
                    let characters =
                        dashboard::selector_entries(&store, |id| match lifecycle.status(id) {
                            Ok(status) => {
                                dashboard::SelectorActivity::from_service(&status.service)
                            }
                            Err(_) => dashboard::SelectorActivity::Unavailable,
                        })?;
                    match dashboard::select_character(characters, &stop)? {
                        Some(id) => id,
                        None => return Ok(()),
                    }
                }
            };
            let provider = LocalProvider::open_default()?;
            provider.refresh(&id)?;
            dashboard::run(&provider, id, Duration::from_millis(refresh_ms), &stop)?;
        }
        Command::NewGuy {
            name,
            race,
            class,
            json,
        } => {
            let mut store = Store::open_default()?;
            let registered = match (name, race, class) {
                (None, None, None) => {
                    let stop = shutdown_flag()?;
                    #[cfg(feature = "enrollment-test-transport")]
                    let completed = {
                        match std::env::var("GYROGNOME_TEST_WIZARD").ok() {
                            Some(script) => {
                                let transport =
                                    reporting::TestEnrollmentTransport::from_environment();
                                let mut activate_online = |draft| {
                                    reporting::enroll_for_test(&mut store, draft, &transport)
                                };
                                newguy_wizard::run_test_script(&script, &mut activate_online)?
                            }
                            None => {
                                let mut activate_online =
                                    |draft| reporting::enroll(&mut store, draft, &HttpsTransport);
                                newguy_wizard::run(&stop, &mut activate_online)?
                            }
                        }
                    };
                    #[cfg(not(feature = "enrollment-test-transport"))]
                    let completed = {
                        let mut activate_online =
                            |draft| reporting::enroll(&mut store, draft, &HttpsTransport);
                        newguy_wizard::run(&stop, &mut activate_online)?
                    };
                    match completed {
                        Some(newguy_wizard::WizardResult::Offline(character)) => {
                            store.register(&character)?
                        }
                        Some(newguy_wizard::WizardResult::Online(registered)) => registered,
                        None => return Ok(()),
                    }
                }
                (Some(name), Some(race), Some(class)) => {
                    let character = newguy::generate_local(
                        &Selection { name, race, class },
                        &crate::ruleset::BUNDLED,
                    )?;
                    store.register(&character)?
                }
                _ => return Err(CliError::PartialCreationInputs),
            };
            print_registration(&registered, json)?;
        }
        #[cfg(feature = "conformance-bridge")]
        Command::ConformanceBridge => conformance_bridge::run_stdio()?,
    }
    Ok(())
}

fn parse_id(value: &str) -> Result<CharacterId, CliError> {
    Ok(CharacterId::parse(value)?)
}

fn compatibility_profile_name(profile: CompatibilityProfile) -> &'static str {
    match profile {
        CompatibilityProfile::Browser => "browser",
        CompatibilityProfile::Desktop644 => "desktop-6.4.4",
    }
}

trait RegistrationView: Serialize {
    fn id(&self) -> &CharacterId;
    fn identity(&self) -> &CharacterIdentity;
}

impl RegistrationView for crate::runtime::ManagedCharacter {
    fn id(&self) -> &CharacterId {
        &self.id
    }

    fn identity(&self) -> &CharacterIdentity {
        &self.identity
    }
}

impl RegistrationView for crate::runtime::RegisteredDesktopCharacter {
    fn id(&self) -> &CharacterId {
        &self.id
    }

    fn identity(&self) -> &CharacterIdentity {
        &self.identity
    }
}

fn print_registration(registered: &impl RegistrationView, json: bool) -> Result<(), CliError> {
    if json {
        println!("{}", to_string_pretty(registered)?);
    } else {
        let identity = registered.identity();
        println!(
            "Registered managed character: {}\n  Name: {}\n  Race: {}\n  Class: {}\n  Level: {}",
            registered.id(),
            identity.name,
            identity.race,
            identity.class,
            identity.level
        );
    }
    Ok(())
}

fn delete_confirmed(input: &mut impl BufRead) -> Result<bool, CliError> {
    let mut response = String::new();
    input
        .read_line(&mut response)
        .map_err(CliError::Confirmation)?;
    Ok(response.trim().eq_ignore_ascii_case("yes"))
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

#[cfg(test)]
mod tests {
    use clap::Parser;
    use std::io::Cursor;

    use super::{Args, Command, delete_confirmed};

    #[test]
    fn profile_commands_require_explicit_values_or_motto_clear() {
        for args in [
            vec!["gyro", "motto", "id"],
            vec!["gyro", "motto", "id", "text", "--clear"],
            vec!["gyro", "guild", "id"],
        ] {
            assert!(Args::try_parse_from(args).is_err());
        }
        assert!(matches!(
            Args::try_parse_from(["gyro", "motto", "id", "--clear"])
                .unwrap()
                .command,
            Command::Motto {
                text: None,
                clear: true,
                ..
            }
        ));
        assert!(
            matches!(Args::try_parse_from(["gyro", "guild", "id", ""]).unwrap().command,
            Command::Guild { designation, .. } if designation.is_empty())
        );
    }

    #[test]
    fn deletion_requires_an_explicit_yes_confirmation() {
        assert!(delete_confirmed(&mut Cursor::new("yes\n")).unwrap());
        assert!(delete_confirmed(&mut Cursor::new("YES\n")).unwrap());
        assert!(!delete_confirmed(&mut Cursor::new("y\n")).unwrap());
        assert!(!delete_confirmed(&mut Cursor::new("no\n")).unwrap());
    }
}
