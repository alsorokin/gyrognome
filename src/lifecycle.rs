//! `systemd --user` lifecycle integration for managed characters.

use std::{
    process::Command,
    thread,
    time::{Duration, Instant},
};

use serde::Serialize;
use thiserror::Error;

use crate::runtime::{CharacterId, CharacterIdentity, StorageError, Store};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceAction {
    Start,
    Stop,
    IsActive,
    MainPid,
    ResetFailed,
}

impl ServiceAction {
    fn systemctl_argument(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::Stop => "stop",
            Self::IsActive => "is-active",
            Self::MainPid => "show",
            Self::ResetFailed => "reset-failed",
        }
    }
}

impl std::fmt::Display for ServiceAction {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.systemctl_argument())
    }
}

#[derive(Debug, Clone)]
pub struct ServiceOutput {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
}

pub trait ServiceRunner {
    fn run(&self, action: ServiceAction, unit: &str) -> Result<ServiceOutput, LifecycleError>;
}

/// The production runner. Tests use [`ServiceRunner`] fakes instead of a real
/// user service manager.
pub struct SystemctlRunner;

impl ServiceRunner for SystemctlRunner {
    fn run(&self, action: ServiceAction, unit: &str) -> Result<ServiceOutput, LifecycleError> {
        let mut command = Command::new("systemctl");
        command.arg("--user").arg(action.systemctl_argument());
        if action == ServiceAction::MainPid {
            command.arg("--property=MainPID").arg("--value");
        }
        let output = command
            .arg(unit)
            .output()
            .map_err(|error| LifecycleError::ManagerUnavailable(error.to_string()))?;
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_owned();
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
        if manager_is_unavailable(&stderr) {
            return Err(LifecycleError::ManagerUnavailable(stderr));
        }
        Ok(ServiceOutput {
            success: output.status.success(),
            stdout,
            stderr,
        })
    }
}

#[derive(Debug, Error)]
pub enum LifecycleError {
    #[error(transparent)]
    Storage(#[from] StorageError),
    #[error("systemd user service manager is unavailable: {0}")]
    ManagerUnavailable(String),
    #[error("systemd user service {action} failed for {unit}: {detail}")]
    ServiceFailure {
        action: ServiceAction,
        unit: String,
        detail: String,
    },
    #[error("systemd user service started but did not acquire character ownership for {0}")]
    RuntimeDidNotStart(CharacterId),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceState {
    Active,
    Inactive,
    Failed,
}

#[derive(Debug, Clone, Serialize)]
pub struct RuntimeStatus {
    pub id: CharacterId,
    pub identity: CharacterIdentity,
    pub service: ServiceState,
    pub runtime_owned: bool,
}

pub struct Lifecycle<'store, Runner> {
    store: &'store Store,
    runner: Runner,
}

impl<'store, Runner: ServiceRunner> Lifecycle<'store, Runner> {
    pub fn new(store: &'store Store, runner: Runner) -> Self {
        Self { store, runner }
    }

    pub fn start(&self, id: &CharacterId) -> Result<(), LifecycleError> {
        self.store.get(id)?;
        if self.store.is_owned(id)? {
            return Err(LifecycleError::Storage(StorageError::AlreadyOwned(
                id.clone(),
            )));
        }
        self.run_required(ServiceAction::Start, id)?;
        let deadline = Instant::now() + Duration::from_secs(1);
        loop {
            let active = self.runner.run(ServiceAction::IsActive, &unit_name(id))?;
            let main_pid = self.runner.run(ServiceAction::MainPid, &unit_name(id))?;
            if active.success
                && main_pid.success
                && let Ok(pid) = main_pid.stdout.parse::<u32>()
                && self.store.is_owned(id)?
                && self.store.owner_process_id(id)? == Some(pid)
            {
                return Ok(());
            }
            if Instant::now() >= deadline {
                return Err(LifecycleError::RuntimeDidNotStart(id.clone()));
            }
            thread::sleep(Duration::from_millis(10));
        }
    }

    pub fn stop(&self, id: &CharacterId) -> Result<(), LifecycleError> {
        self.store.get(id)?;
        self.run_required(ServiceAction::Stop, id)
    }

    /// Clears a failed service state and starts a fresh worker. Worker locking
    /// still rejects recovery if another local process owns the character.
    pub fn recover(&self, id: &CharacterId) -> Result<(), LifecycleError> {
        self.store.get(id)?;
        self.run_required(ServiceAction::ResetFailed, id)?;
        self.start(id)
    }

    pub fn status(&self, id: &CharacterId) -> Result<RuntimeStatus, LifecycleError> {
        let character = self.store.get(id)?;
        let output = self.runner.run(ServiceAction::IsActive, &unit_name(id))?;
        Ok(RuntimeStatus {
            id: id.clone(),
            identity: character.identity,
            service: if output.success {
                ServiceState::Active
            } else if output.stdout == "failed" {
                ServiceState::Failed
            } else {
                ServiceState::Inactive
            },
            runtime_owned: self.store.is_owned(id)?,
        })
    }

    fn run_required(&self, action: ServiceAction, id: &CharacterId) -> Result<(), LifecycleError> {
        let unit = unit_name(id);
        let output = self.runner.run(action, &unit)?;
        if output.success {
            Ok(())
        } else {
            let detail = if output.stderr.is_empty() {
                output.stdout
            } else {
                output.stderr
            };
            Err(LifecycleError::ServiceFailure {
                action,
                unit,
                detail,
            })
        }
    }
}

fn unit_name(id: &CharacterId) -> String {
    format!("gyrognome@{id}.service")
}

fn manager_is_unavailable(stderr: &str) -> bool {
    [
        "Failed to connect to bus",
        "No medium found",
        "Failed to connect to user scope bus",
    ]
    .iter()
    .any(|message| stderr.contains(message))
}

#[cfg(test)]
mod tests {
    use std::{
        collections::VecDeque,
        fs,
        path::{Path, PathBuf},
    };

    use base64::{Engine, engine::general_purpose::STANDARD};
    use uuid::Uuid;

    use super::*;
    use crate::{runtime::Store, save};

    struct TestDirectory(PathBuf);

    impl TestDirectory {
        fn new() -> Self {
            let path = Path::new("target")
                .join("gyrognome-lifecycle-tests")
                .join(Uuid::new_v4().to_string());
            fs::create_dir_all(&path).unwrap();
            Self(path)
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    struct FakeRunner {
        results: std::cell::RefCell<VecDeque<Result<ServiceOutput, LifecycleError>>>,
    }

    impl FakeRunner {
        fn with(result: Result<ServiceOutput, LifecycleError>) -> Self {
            Self {
                results: std::cell::RefCell::new(VecDeque::from([result])),
            }
        }
    }

    impl ServiceRunner for FakeRunner {
        fn run(
            &self,
            _action: ServiceAction,
            _unit: &str,
        ) -> Result<ServiceOutput, LifecycleError> {
            self.results.borrow_mut().pop_front().unwrap()
        }
    }

    fn registered_store() -> (TestDirectory, Store, CharacterId) {
        let directory = TestDirectory::new();
        let mut store = Store::open_at(&directory.0).unwrap();
        let character = save::import_text(
            &STANDARD.encode(include_str!("../tests/fixtures/reference-save.json")),
        )
        .unwrap();
        let id = store.register(&character).unwrap().id;
        (directory, store, id)
    }

    #[test]
    fn status_distinguishes_active_and_inactive_services() {
        let (_directory, store, id) = registered_store();
        let active = Lifecycle::new(
            &store,
            FakeRunner::with(Ok(ServiceOutput {
                success: true,
                stdout: "active".to_owned(),
                stderr: String::new(),
            })),
        )
        .status(&id)
        .unwrap();
        assert_eq!(active.service, ServiceState::Active);
        assert!(!active.runtime_owned);
        assert_eq!(active.identity.name, "Reference Hero");

        let inactive = Lifecycle::new(
            &store,
            FakeRunner::with(Ok(ServiceOutput {
                success: false,
                stdout: "inactive".to_owned(),
                stderr: String::new(),
            })),
        )
        .status(&id)
        .unwrap();
        assert_eq!(inactive.service, ServiceState::Inactive);

        let failed = Lifecycle::new(
            &store,
            FakeRunner::with(Ok(ServiceOutput {
                success: false,
                stdout: "failed".to_owned(),
                stderr: String::new(),
            })),
        )
        .status(&id)
        .unwrap();
        assert_eq!(failed.service, ServiceState::Failed);
    }

    #[test]
    fn reports_an_unavailable_user_service_manager() {
        let (_directory, store, id) = registered_store();
        let error = Lifecycle::new(
            &store,
            FakeRunner::with(Err(LifecycleError::ManagerUnavailable(
                "Failed to connect to bus".to_owned(),
            ))),
        )
        .status(&id)
        .unwrap_err();
        assert!(matches!(error, LifecycleError::ManagerUnavailable(_)));
    }

    #[test]
    fn reports_service_failures_and_recovery_runs_both_steps() {
        let (_directory, store, id) = registered_store();
        let failure = Lifecycle::new(
            &store,
            FakeRunner::with(Ok(ServiceOutput {
                success: false,
                stdout: String::new(),
                stderr: "unit failed".to_owned(),
            })),
        )
        .start(&id)
        .unwrap_err();
        assert!(matches!(
            failure,
            LifecycleError::ServiceFailure {
                action: ServiceAction::Start,
                ..
            }
        ));

        let runner = FakeRunner {
            results: std::cell::RefCell::new(VecDeque::from([
                Ok(ServiceOutput {
                    success: true,
                    stdout: String::new(),
                    stderr: String::new(),
                }),
                Ok(ServiceOutput {
                    success: false,
                    stdout: String::new(),
                    stderr: "worker rejected startup".to_owned(),
                }),
            ])),
        };
        let error = Lifecycle::new(&store, runner).recover(&id).unwrap_err();
        assert!(matches!(
            error,
            LifecycleError::ServiceFailure {
                action: ServiceAction::Start,
                ..
            }
        ));
    }

    #[test]
    fn rejects_lifecycle_start_when_a_worker_already_owns_the_character() {
        let (_directory, store, id) = registered_store();
        let worker =
            crate::runtime::Worker::start(Store::open_at(store.data_root()).unwrap(), id.clone())
                .unwrap();

        let error = Lifecycle::new(
            &store,
            FakeRunner::with(Ok(ServiceOutput {
                success: true,
                stdout: String::new(),
                stderr: String::new(),
            })),
        )
        .start(&id)
        .unwrap_err();

        assert!(matches!(
            error,
            LifecycleError::Storage(StorageError::AlreadyOwned(owned)) if owned == id
        ));
        drop(worker);
    }
}
