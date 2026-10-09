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
    Enable,
    Disable,
    IsEnabled,
}

impl ServiceAction {
    fn systemctl_argument(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::Stop => "stop",
            Self::IsActive => "is-active",
            Self::MainPid => "show",
            Self::ResetFailed => "reset-failed",
            Self::Enable => "enable",
            Self::Disable => "disable",
            Self::IsEnabled => "is-enabled",
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
        if matches!(
            action,
            ServiceAction::Enable | ServiceAction::Disable | ServiceAction::IsEnabled
        ) {
            command.arg("--no-reload").arg("--root=/");
        }
        command.env("LC_ALL", "C");
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
    #[error("managed character {0} is running; stop it before deleting")]
    CharacterRunning(CharacterId),
    #[error("character deletion failed after autostart was disabled: {0}")]
    RemovalAfterCleanup(StorageError),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceState {
    Active,
    Inactive,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AutostartState {
    Enabled,
    Disabled,
    Unavailable,
}

impl AutostartState {
    pub fn label(self) -> &'static str {
        match self {
            Self::Enabled => "On",
            Self::Disabled => "Off",
            Self::Unavailable => "Unavailable",
        }
    }
}

#[derive(Debug, Clone)]
pub struct AutostartStatus {
    pub state: AutostartState,
    pub diagnostic: Option<String>,
}

impl AutostartStatus {
    pub fn unavailable(detail: impl std::fmt::Display) -> Self {
        Self {
            state: AutostartState::Unavailable,
            diagnostic: Some(detail.to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RuntimeStatus {
    pub id: CharacterId,
    pub identity: CharacterIdentity,
    pub service: ServiceState,
    pub runtime_owned: bool,
    pub autostart: AutostartState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub autostart_diagnostic: Option<String>,
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
        self.store.identity(id)?;
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
        self.store.identity(id)?;
        self.run_required(ServiceAction::Stop, id)
    }

    /// Clears a failed service state and starts a fresh worker. Worker locking
    /// still rejects recovery if another local process owns the character.
    pub fn recover(&self, id: &CharacterId) -> Result<(), LifecycleError> {
        self.store.identity(id)?;
        self.run_required(ServiceAction::ResetFailed, id)?;
        self.start(id)
    }

    pub fn status(&self, id: &CharacterId) -> Result<RuntimeStatus, LifecycleError> {
        let identity = self.store.identity(id)?;
        let output = self.runner.run(ServiceAction::IsActive, &unit_name(id))?;
        let autostart = self.autostart(id)?;
        Ok(RuntimeStatus {
            id: id.clone(),
            identity,
            service: if output.success {
                ServiceState::Active
            } else if output.stdout == "failed" {
                ServiceState::Failed
            } else {
                ServiceState::Inactive
            },
            runtime_owned: self.store.is_owned(id)?,
            autostart: autostart.state,
            autostart_diagnostic: autostart.diagnostic,
        })
    }

    pub fn autostart(&self, id: &CharacterId) -> Result<AutostartStatus, LifecycleError> {
        self.store.identity(id)?;
        Ok(
            match self.runner.run(ServiceAction::IsEnabled, &unit_name(id)) {
                Ok(output) => parse_autostart(output, id),
                Err(error) => AutostartStatus::unavailable(error),
            },
        )
    }

    pub fn set_autostart(&self, id: &CharacterId, enabled: bool) -> Result<(), LifecycleError> {
        self.store.identity(id)?;
        self.run_required(
            if enabled {
                ServiceAction::Enable
            } else {
                ServiceAction::Disable
            },
            id,
        )?;
        let status = self.autostart(id)?;
        let expected = if enabled {
            AutostartState::Enabled
        } else {
            AutostartState::Disabled
        };
        if status.state != expected {
            return Err(LifecycleError::ServiceFailure {
                action: ServiceAction::IsEnabled,
                unit: unit_name(id),
                detail: status.diagnostic.unwrap_or_else(|| {
                    format!(
                        "expected {}, observed {}",
                        expected.label(),
                        status.state.label()
                    )
                }),
            });
        }
        Ok(())
    }

    fn run_required(&self, action: ServiceAction, id: &CharacterId) -> Result<(), LifecycleError> {
        let unit = unit_name(id);
        let output = self.runner.run(action, &unit)?;
        if output.success {
            Ok(())
        } else {
            let detail = output_detail(output);
            Err(LifecycleError::ServiceFailure {
                action,
                unit,
                detail,
            })
        }
    }
}

fn parse_autostart(output: ServiceOutput, id: &CharacterId) -> AutostartStatus {
    let state = match (output.stdout.as_str(), output.success) {
        ("enabled", true) => AutostartState::Enabled,
        ("disabled" | "enabled-runtime", _) if output.stderr.is_empty() => AutostartState::Disabled,
        _ => {
            return AutostartStatus::unavailable(format!(
                "Could not determine persistent autostart for {}: {}. Check the installed user-service template and systemctl --user is-enabled.",
                unit_name(id),
                output_detail(output),
            ));
        }
    };
    AutostartStatus {
        state,
        diagnostic: None,
    }
}

fn output_detail(output: ServiceOutput) -> String {
    if !output.stderr.is_empty() {
        output.stderr
    } else if !output.stdout.is_empty() {
        output.stdout
    } else {
        "systemctl returned no diagnostic".to_owned()
    }
}

/// Keeps character ownership locked across unit cleanup and atomic data removal.
pub fn delete_character(
    store: &mut Store,
    runner: &impl ServiceRunner,
    id: &CharacterId,
) -> Result<(), LifecycleError> {
    store.identity(id)?;
    let mut cleaned = false;
    let result = store.remove_with_cleanup(id, || {
        let unit = unit_name(id);
        match runner.run(ServiceAction::IsActive, &unit) {
            Ok(output)
                if output.success
                    || matches!(
                        output.stdout.as_str(),
                        "activating" | "deactivating" | "reloading"
                    ) =>
            {
                return Err(LifecycleError::CharacterRunning(id.clone()));
            }
            Ok(output) if matches!(output.stdout.as_str(), "inactive" | "failed" | "unknown") => {}
            Err(LifecycleError::ManagerUnavailable(_)) => {
                // The held ownership lock proves that no worker is running.
            }
            Ok(output) => {
                return Err(LifecycleError::ServiceFailure {
                    action: ServiceAction::IsActive,
                    unit,
                    detail: output_detail(output),
                });
            }
            Err(error) => return Err(error),
        }
        // Disable also removes dangling startup links when the template is gone.
        let output = runner.run(ServiceAction::Disable, &unit)?;
        let missing = format!("Failed to disable unit: Unit {unit} does not exist");
        let absent_unit = !output.success
            && output.stderr.lines().any(|line| line == missing)
            && output
                .stderr
                .lines()
                .all(|line| line == missing || line.starts_with("Removed "));
        if !output.success && !absent_unit {
            return Err(LifecycleError::ServiceFailure {
                action: ServiceAction::Disable,
                unit,
                detail: output_detail(output),
            });
        }
        cleaned = true;
        let output = runner.run(ServiceAction::IsEnabled, &unit)?;
        if matches!(output.stdout.as_str(), "disabled" | "not-found") && output.stderr.is_empty() {
            Ok(())
        } else {
            Err(LifecycleError::ServiceFailure {
                action: ServiceAction::IsEnabled,
                unit,
                detail: output_detail(output),
            })
        }
    });
    match result {
        Err(LifecycleError::Storage(error)) if cleaned => {
            Err(LifecycleError::RemovalAfterCleanup(error))
        }
        other => other,
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
                results: std::cell::RefCell::new(VecDeque::from([
                    result,
                    Ok(ServiceOutput {
                        success: false,
                        stdout: "disabled".to_owned(),
                        stderr: String::new(),
                    }),
                ])),
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

    struct RecordingRunner {
        results: std::cell::RefCell<VecDeque<Result<ServiceOutput, LifecycleError>>>,
        actions: std::cell::RefCell<Vec<ServiceAction>>,
    }

    impl ServiceRunner for &RecordingRunner {
        fn run(&self, action: ServiceAction, _: &str) -> Result<ServiceOutput, LifecycleError> {
            self.actions.borrow_mut().push(action);
            self.results
                .borrow_mut()
                .pop_front()
                .expect("unexpected service action")
        }
    }

    fn output(success: bool, state: &str) -> Result<ServiceOutput, LifecycleError> {
        Ok(ServiceOutput {
            success,
            stdout: state.to_owned(),
            stderr: String::new(),
        })
    }

    fn recording(results: Vec<Result<ServiceOutput, LifecycleError>>) -> RecordingRunner {
        RecordingRunner {
            results: std::cell::RefCell::new(results.into()),
            actions: std::cell::RefCell::new(Vec::new()),
        }
    }

    #[test]
    fn persistent_unit_states_and_failures_are_distinct() {
        let (_directory, store, id) = registered_store();
        for (state, success, expected) in [
            ("enabled", true, AutostartState::Enabled),
            ("disabled", false, AutostartState::Disabled),
            ("enabled-runtime", true, AutostartState::Disabled),
            ("masked", false, AutostartState::Unavailable),
            ("static", true, AutostartState::Unavailable),
            ("indirect", true, AutostartState::Unavailable),
            ("not-found", false, AutostartState::Unavailable),
            ("enabled", false, AutostartState::Unavailable),
            ("", false, AutostartState::Unavailable),
        ] {
            let runner = recording(vec![output(success, state)]);
            let status = Lifecycle::new(&store, &runner).autostart(&id).unwrap();
            assert_eq!(status.state, expected, "{state}");
            assert_eq!(
                status.diagnostic.is_some(),
                expected == AutostartState::Unavailable
            );
        }
        let runner = recording(vec![Err(LifecycleError::ManagerUnavailable(
            "no systemctl".to_owned(),
        ))]);
        let status = Lifecycle::new(&store, &runner).autostart(&id).unwrap();
        assert_eq!(status.state, AutostartState::Unavailable);
        assert!(status.diagnostic.unwrap().contains("no systemctl"));
    }

    #[test]
    fn repeated_autostart_settings_never_start_or_stop_a_worker() {
        let (_directory, store, id) = registered_store();
        for enabled in [true, false] {
            let state = if enabled { "enabled" } else { "disabled" };
            let runner = recording(vec![
                output(true, ""),
                output(enabled, state),
                output(true, ""),
                output(enabled, state),
            ]);
            let lifecycle = Lifecycle::new(&store, &runner);
            lifecycle.set_autostart(&id, enabled).unwrap();
            lifecycle.set_autostart(&id, enabled).unwrap();
            let action = if enabled {
                ServiceAction::Enable
            } else {
                ServiceAction::Disable
            };
            assert_eq!(
                *runner.actions.borrow(),
                [
                    action,
                    ServiceAction::IsEnabled,
                    action,
                    ServiceAction::IsEnabled
                ]
            );
            assert!(!store.is_owned(&id).unwrap());
        }
    }

    #[test]
    fn setting_autostart_reports_command_and_verification_failures() {
        let (_directory, store, id) = registered_store();
        for results in [
            vec![output(false, "permission denied")],
            vec![output(true, ""), output(false, "disabled")],
        ] {
            let runner = recording(results);
            assert!(
                Lifecycle::new(&store, &runner)
                    .set_autostart(&id, true)
                    .is_err()
            );
            assert!(store.identity(&id).is_ok());
        }
        let runner = recording(Vec::new());
        assert!(
            Lifecycle::new(&store, &runner)
                .set_autostart(&CharacterId::new(), true)
                .is_err()
        );
        assert!(runner.actions.borrow().is_empty());
    }

    #[test]
    fn autostart_failure_preserves_runtime_status() {
        let (_directory, store, id) = registered_store();
        let runner = recording(vec![output(false, "inactive"), output(false, "masked")]);
        let status = Lifecycle::new(&store, &runner).status(&id).unwrap();
        assert_eq!(status.service, ServiceState::Inactive);
        assert!(!status.runtime_owned);
        assert_eq!(status.autostart, AutostartState::Unavailable);
        assert!(status.autostart_diagnostic.is_some());
    }

    #[test]
    fn deletion_cleans_startup_before_removing_data() {
        let (_directory, mut store, id) = registered_store();
        let runner = recording(vec![
            output(false, "inactive"),
            output(true, ""),
            output(false, "disabled"),
        ]);
        delete_character(&mut store, &(&runner), &id).unwrap();
        assert!(matches!(
            store.identity(&id),
            Err(StorageError::NotFound(_))
        ));
        assert_eq!(
            *runner.actions.borrow(),
            [
                ServiceAction::IsActive,
                ServiceAction::Disable,
                ServiceAction::IsEnabled,
            ]
        );
    }

    #[test]
    fn deletion_failures_and_active_workers_preserve_data_and_preferences() {
        let (_directory, mut store, id) = registered_store();
        let runner = recording(vec![output(true, "active")]);
        assert!(matches!(
            delete_character(&mut store, &(&runner), &id),
            Err(LifecycleError::CharacterRunning(_))
        ));
        assert_eq!(*runner.actions.borrow(), [ServiceAction::IsActive]);

        let runner = recording(vec![
            output(false, "inactive"),
            output(false, "permission denied"),
        ]);
        assert!(delete_character(&mut store, &(&runner), &id).is_err());
        assert!(store.identity(&id).is_ok());
        let worker =
            crate::runtime::Worker::start(Store::open_at(store.data_root()).unwrap(), id.clone())
                .unwrap();
        let runner = recording(Vec::new());
        assert!(matches!(
            delete_character(&mut store, &(&runner), &id),
            Err(LifecycleError::Storage(StorageError::AlreadyOwned(_)))
        ));
        assert!(runner.actions.borrow().is_empty());
        drop(worker);
    }

    #[test]
    fn deletion_without_template_or_running_user_manager_is_supported() {
        let (_directory, mut store, id) = registered_store();
        let runner = recording(vec![
            Err(LifecycleError::ManagerUnavailable("no user bus".to_owned())),
            output(true, ""),
            output(false, "not-found"),
        ]);
        delete_character(&mut store, &(&runner), &id).unwrap();
    }

    #[test]
    fn deletion_reports_disabled_autostart_after_atomic_removal_failure() {
        let (_directory, mut store, id) = registered_store();
        store.inject_next_remove_failure();
        let runner = recording(vec![
            output(false, "inactive"),
            output(true, ""),
            output(false, "disabled"),
        ]);
        let error = delete_character(&mut store, &(&runner), &id).unwrap_err();
        assert!(matches!(error, LifecycleError::RemovalAfterCleanup(_)));
        assert!(error.to_string().contains("autostart was disabled"));
        assert!(store.identity(&id).is_ok());
    }
}
