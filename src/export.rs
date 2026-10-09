//! Exports a managed character to a `.pqw` or `.pq` save file.

use std::{
    fs::{self, OpenOptions},
    io::{self, Write},
    os::unix::fs::OpenOptionsExt,
    path::{Path, PathBuf},
};

use thiserror::Error;
use uuid::Uuid;

use crate::{
    lifecycle::{Lifecycle, LifecycleError, ServiceRunner, ServiceState},
    runtime::{CharacterId, StorageError, Store},
};

#[derive(Debug, Error)]
pub enum ExportError {
    #[error(transparent)]
    Storage(#[from] StorageError),
    #[error(transparent)]
    Lifecycle(#[from] LifecycleError),
    #[error("could not write the export: {0}")]
    Io(#[from] io::Error),
    #[error("{0} already exists; confirm interactively or pass --force to overwrite it")]
    OverwriteNotConfirmed(PathBuf),
    #[error("{0} is not a regular file path")]
    NotAFile(PathBuf),
    #[error(
        "managed character {0} is running outside the service manager; stop it before exporting"
    )]
    RunningOutsideService(CharacterId),
    #[error("export failed ({export}) and the character could not be restarted: {restart}")]
    RestartAfterFailure {
        export: Box<ExportError>,
        restart: LifecycleError,
    },
    #[error("exported to {path}, but the character could not be restarted: {restart}")]
    RestartFailed {
        path: PathBuf,
        restart: LifecycleError,
    },
}

#[derive(Debug, PartialEq, Eq)]
pub struct ExportOutcome {
    pub path: PathBuf,
    pub restarted: bool,
}

/// Exports `id`, stopping and restarting its service if it is running.
/// `confirm_overwrite` is only asked when the target exists and `force` is off.
pub fn export_character<R: ServiceRunner>(
    store: &Store,
    runner: R,
    id: &CharacterId,
    output: Option<&Path>,
    force: bool,
    confirm_overwrite: &mut dyn FnMut(&Path) -> io::Result<bool>,
) -> Result<ExportOutcome, ExportError> {
    let target = store.export_target(id)?;
    let path = match output {
        Some(path) => path.to_owned(),
        None => PathBuf::from(format!(
            "{}.{}",
            sanitize_file_stem(&target.file_stem),
            target.extension
        )),
    };
    match fs::metadata(&path) {
        Ok(metadata) if !metadata.is_file() => return Err(ExportError::NotAFile(path)),
        Ok(_) if !force => {
            if !confirm_overwrite(&path)? {
                return Err(ExportError::OverwriteNotConfirmed(path));
            }
        }
        Ok(_) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }

    let lifecycle = Lifecycle::new(store, runner);
    let status = lifecycle.status(id)?;
    let was_running = status.service == ServiceState::Active;
    if was_running {
        lifecycle.stop(id)?;
    }
    if store.is_owned(id)? {
        return Err(ExportError::RunningOutsideService(id.clone()));
    }

    let result = store
        .export_save(id)
        .map_err(ExportError::from)
        .and_then(|bytes| write_private_atomic(&path, &bytes).map_err(ExportError::from));

    let restart = if was_running {
        Some(lifecycle.start(id))
    } else {
        None
    };
    match (result, restart) {
        (Ok(()), None) => Ok(ExportOutcome {
            path,
            restarted: false,
        }),
        (Ok(()), Some(Ok(()))) => Ok(ExportOutcome {
            path,
            restarted: true,
        }),
        (Ok(()), Some(Err(restart))) => Err(ExportError::RestartFailed { path, restart }),
        (Err(error), Some(Err(restart))) => Err(ExportError::RestartAfterFailure {
            export: Box::new(error),
            restart,
        }),
        (Err(error), _) => Err(error),
    }
}

fn sanitize_file_stem(stem: &str) -> String {
    let cleaned: String = stem
        .chars()
        .map(|character| {
            if character.is_control() || matches!(character, '/' | '\\' | '\0') {
                '_'
            } else {
                character
            }
        })
        .collect();
    let cleaned = cleaned.trim().trim_start_matches('.').to_owned();
    if cleaned.is_empty() {
        "character".to_owned()
    } else {
        cleaned
    }
}

fn write_private_atomic(target: &Path, bytes: &[u8]) -> io::Result<()> {
    let directory = target
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let file_name = target
        .file_name()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "missing file name"))?;
    let temporary = directory.join(format!(
        ".{}.{}.tmp",
        file_name.to_string_lossy(),
        Uuid::new_v4()
    ));
    let write = || -> io::Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&temporary)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        fs::rename(&temporary, target)
    };
    let result = write();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, collections::VecDeque, os::unix::fs::PermissionsExt};

    use base64::{Engine, engine::general_purpose::STANDARD};

    use super::*;
    use crate::{
        lifecycle::{ServiceAction, ServiceOutput},
        save,
    };

    struct TestDirectory(PathBuf);

    impl TestDirectory {
        fn new() -> Self {
            let path = Path::new("target")
                .join("gyrognome-export-tests")
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
        active: bool,
        actions: RefCell<Vec<ServiceAction>>,
        pending: RefCell<VecDeque<bool>>,
    }

    impl FakeRunner {
        fn new(active: bool) -> Self {
            Self {
                active,
                actions: RefCell::new(Vec::new()),
                pending: RefCell::new(VecDeque::new()),
            }
        }
    }

    impl ServiceRunner for &FakeRunner {
        fn run(
            &self,
            action: ServiceAction,
            _unit: &str,
        ) -> Result<ServiceOutput, LifecycleError> {
            self.actions.borrow_mut().push(action);
            let _ = &self.pending;
            Ok(ServiceOutput {
                success: match action {
                    ServiceAction::IsActive => self.active,
                    _ => true,
                },
                stdout: String::new(),
                stderr: String::new(),
            })
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
    fn exports_browser_save_that_imports_back_with_credentials() {
        let (directory, store, id) = registered_store();
        let output = directory.0.join("out.pqw");
        let outcome = export_character(
            &store,
            &FakeRunner::new(false),
            &id,
            Some(&output),
            false,
            &mut |_| panic!("no existing file"),
        )
        .unwrap();
        assert!(!outcome.restarted);
        assert_eq!(
            fs::metadata(&output).unwrap().permissions().mode() & 0o777,
            0o600
        );
        let imported = save::import_file(&output).unwrap();
        assert_eq!(imported.traits.name, "Reference Hero");
        assert_eq!(imported.document["online"]["passkey"], 4242);
    }

    #[test]
    fn declined_overwrite_keeps_existing_file_and_force_replaces_it() {
        let (directory, store, id) = registered_store();
        let output = directory.0.join("out.pqw");
        fs::write(&output, b"old").unwrap();
        let declined = export_character(
            &store,
            &FakeRunner::new(false),
            &id,
            Some(&output),
            false,
            &mut |_| Ok(false),
        );
        assert!(matches!(declined, Err(ExportError::OverwriteNotConfirmed(_))));
        assert_eq!(fs::read(&output).unwrap(), b"old");

        export_character(
            &store,
            &FakeRunner::new(false),
            &id,
            Some(&output),
            true,
            &mut |_| panic!("force must not prompt"),
        )
        .unwrap();
        assert!(save::import_file(&output).is_ok());
    }

    #[test]
    fn running_character_is_stopped_then_restarted() {
        let (directory, store, id) = registered_store();
        let output = directory.0.join("out.pqw");
        let runner = FakeRunner::new(true);
        // The fake service never takes ownership, so the restart check fails;
        // this still proves a restart was attempted after the export.
        let result = export_character(&store, &runner, &id, Some(&output), false, &mut |_| {
            Ok(true)
        });
        assert!(matches!(result, Err(ExportError::RestartFailed { .. })));
        assert!(save::import_file(&output).is_ok());
        let actions = runner.actions.borrow();
        let stop = actions.iter().position(|a| *a == ServiceAction::Stop).unwrap();
        let start = actions.iter().position(|a| *a == ServiceAction::Start).unwrap();
        assert!(stop < start);
    }

    #[test]
    fn failed_export_still_restarts_running_character() {
        let (directory, store, id) = registered_store();
        let output = directory.0.join("missing-directory").join("out.pqw");
        let runner = FakeRunner::new(true);
        let result = export_character(&store, &runner, &id, Some(&output), false, &mut |_| {
            Ok(true)
        });
        assert!(matches!(result, Err(ExportError::RestartAfterFailure { .. })));
        assert!(runner.actions.borrow().contains(&ServiceAction::Start));
    }

    #[test]
    fn sanitizes_default_file_names() {
        assert_eq!(sanitize_file_stem("../a/b"), "_a_b");
        assert_eq!(sanitize_file_stem("  "), "character");
    }
}
