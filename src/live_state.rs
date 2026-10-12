//! Credential-safe live runtime state published for local observers.
//!
//! A worker atomically replaces one small file per character, normally in the
//! RAM-backed per-user runtime directory, so the dashboard can follow progress
//! between the infrequent durable database commits.

use std::{
    env,
    fs::{self, DirBuilder, OpenOptions},
    io::{self, Write},
    os::unix::fs::{DirBuilderExt, OpenOptionsExt},
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{compatibility::CompatibilityProfile, rested::RestedState};

const VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct LiveState {
    pub version: u32,
    pub pid: u32,
    pub profile: CompatibilityProfile,
    pub state: Value,
    pub rested: RestedState,
}

impl LiveState {
    pub fn new(profile: CompatibilityProfile, state: Value, rested: RestedState) -> Self {
        Self {
            version: VERSION,
            pid: std::process::id(),
            profile,
            state,
            rested,
        }
    }
}

/// Prefers `$XDG_RUNTIME_DIR/gyrognome`, falling back to `<data_root>/live`.
pub(crate) fn default_directory(data_root: &Path) -> PathBuf {
    env::var_os("XDG_RUNTIME_DIR")
        .filter(|value| !value.is_empty())
        .map(|value| PathBuf::from(value).join("gyrognome"))
        .unwrap_or_else(|| data_root.join("live"))
}

fn path(directory: &Path, id: &str) -> PathBuf {
    directory.join(format!("{id}.live.json"))
}

pub(crate) fn publish(directory: &Path, id: &str, live: &LiveState) -> io::Result<()> {
    DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(directory)?;
    let temporary = directory.join(format!("{id}.live.json.{}.tmp", live.pid));
    let result = (|| {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(&temporary)?;
        file.write_all(&serde_json::to_vec(live)?)?;
        drop(file);
        fs::rename(&temporary, path(directory, id))
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

pub(crate) fn read(directory: &Path, id: &str) -> Option<LiveState> {
    let bytes = fs::read(path(directory, id)).ok()?;
    serde_json::from_slice::<LiveState>(&bytes)
        .ok()
        .filter(|live| live.version == VERSION)
}

pub(crate) fn remove(directory: &Path, id: &str) {
    let _ = fs::remove_file(path(directory, id));
}

#[cfg(test)]
mod tests {
    use std::os::unix::fs::PermissionsExt;

    use super::*;

    #[test]
    fn publishes_atomically_with_private_permissions_and_round_trips() {
        let directory = env::temp_dir()
            .join(format!("gyrognome-live-{}", uuid::Uuid::new_v4()))
            .join("live");
        let rested = RestedState::empty(1_000);
        let live = LiveState::new(
            CompatibilityProfile::Browser,
            serde_json::json!({ "x": 1 }),
            rested,
        );
        publish(&directory, "abc", &live).unwrap();
        publish(&directory, "abc", &live).unwrap();

        let read_back = read(&directory, "abc").unwrap();
        assert_eq!(read_back.pid, std::process::id());
        assert_eq!(read_back.state, serde_json::json!({ "x": 1 }));
        assert_eq!(read_back.rested, rested);
        let mode = |path: &Path| fs::metadata(path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode(&directory), 0o700);
        assert_eq!(mode(&path(&directory, "abc")), 0o600);
        assert_eq!(fs::read_dir(&directory).unwrap().count(), 1);

        remove(&directory, "abc");
        assert!(read(&directory, "abc").is_none());
        fs::remove_dir_all(directory.parent().unwrap()).unwrap();
    }
}
