//! Local-only persistence for managed characters.
//!
//! This module deliberately owns filesystem and SQLite concerns outside the
//! pure `simulation` module. Canonical state is stored separately from the
//! original browser document so a future exporter can retain unknown fields.

use std::{
    env, fs,
    fs::{File, OpenOptions},
    io::{ErrorKind, Write},
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use directories::BaseDirs;
use fs2::FileExt;
use rusqlite::{Connection, TransactionBehavior, params};
use serde::{Serialize, Serializer};
use thiserror::Error;
use uuid::Uuid;

use crate::{
    reporting::{HttpsTransport, ReportTransport},
    state::{Character, OnlineProfile},
};
use rusqlite::OptionalExtension;
#[cfg(test)]
use serde_json::Value;

const DATABASE_FILENAME: &str = "characters.sqlite3";
const DATABASE_SCHEMA_VERSION: i64 = 2;
pub const CANONICAL_STATE_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CharacterId(Uuid);

impl CharacterId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn parse(value: &str) -> Result<Self, StorageError> {
        Uuid::parse_str(value)
            .map(Self)
            .map_err(|_| StorageError::InvalidCharacterId(value.to_owned()))
    }
}

impl Default for CharacterId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for CharacterId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
    }
}

impl std::str::FromStr for CharacterId {
    type Err = StorageError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::parse(value)
    }
}

impl Serialize for CharacterId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CharacterIdentity {
    pub name: String,
    pub race: String,
    pub class: String,
    pub level: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ManagedCharacter {
    pub id: CharacterId,
    pub identity: CharacterIdentity,
    pub state_version: u32,
    pub created_at_unix_ms: i64,
    pub updated_at_unix_ms: i64,
    pub state: Character,
}

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("could not determine the invoking user's data directory")]
    DataHomeUnavailable,
    #[error("managed character identifier is invalid: {0}")]
    InvalidCharacterId(String),
    #[error("could not access local character data: {0}")]
    Io(#[from] std::io::Error),
    #[error("could not access local character database: {0}")]
    Sql(#[from] rusqlite::Error),
    #[error("could not serialize or read canonical character state: {0}")]
    StateJson(#[source] serde_json::Error),
    #[error("local character database schema version {0} is newer than this binary supports")]
    UnsupportedSchema(i64),
    #[error("managed character {0} was not found")]
    NotFound(CharacterId),
    #[error("managed character {0} is already running")]
    AlreadyOwned(CharacterId),
    #[error("managed character is not eligible for reporting")]
    ReportingIneligible,
    #[error("numeric value is outside SQLite's signed integer range: {0}")]
    IntegerOutOfRange(&'static str),
    #[cfg(test)]
    #[error("injected storage failure")]
    InjectedFailure,
}

/// Returns the per-user directory containing the runtime database and locks.
///
/// `XDG_DATA_HOME` takes precedence. On Linux, `BaseDirs` falls back to
/// `~/.local/share` when that variable is unset.
pub fn data_root() -> Result<PathBuf, StorageError> {
    if let Some(value) = env::var_os("XDG_DATA_HOME").filter(|value| !value.is_empty()) {
        return Ok(PathBuf::from(value).join("gyrognome"));
    }
    BaseDirs::new()
        .map(|directories| directories.data_local_dir().join("gyrognome"))
        .ok_or(StorageError::DataHomeUnavailable)
}

#[cfg(test)]
fn data_root_from(
    xdg_data_home: Option<&Path>,
    home: Option<&Path>,
) -> Result<PathBuf, StorageError> {
    if let Some(data_home) = xdg_data_home.filter(|path| !path.as_os_str().is_empty()) {
        return Ok(data_home.join("gyrognome"));
    }
    home.map(|home| home.join(".local/share/gyrognome"))
        .ok_or(StorageError::DataHomeUnavailable)
}

pub struct Store {
    data_root: PathBuf,
    connection: Connection,
    #[cfg(test)]
    fail_next_update: bool,
    #[cfg(test)]
    fail_next_profile_update: bool,
    #[cfg(test)]
    fail_next_remove: bool,
}

/// An advisory lock held for a managed character's worker lifetime.
pub struct CharacterLock {
    _file: File,
}

/// A short-lived advisory lock that serializes online requests for one character.
pub(crate) struct OnlineActionLock {
    _file: File,
}

pub(crate) struct ReportingTarget {
    pub(crate) identity: CharacterIdentity,
    pub(crate) state: Character,
    pub(crate) passkey: i32,
    _online_action_lock: OnlineActionLock,
}

impl Store {
    pub fn open_default() -> Result<Self, StorageError> {
        Self::open_at(data_root()?)
    }

    /// Opens the store rooted at `data_root`. This is primarily useful to
    /// embedders that need an explicitly selected local data directory.
    pub fn open_at(data_root: impl AsRef<Path>) -> Result<Self, StorageError> {
        let data_root = data_root.as_ref().to_path_buf();
        fs::create_dir_all(&data_root)?;
        restrict_permissions(&data_root, 0o700)?;
        let database_path = data_root.join(DATABASE_FILENAME);
        let mut connection = Connection::open(&database_path)?;
        restrict_permissions(&database_path, 0o600)?;
        connection.busy_timeout(Duration::from_secs(5))?;
        initialize_schema(&mut connection)?;
        Ok(Self {
            data_root,
            connection,
            #[cfg(test)]
            fail_next_update: false,
            #[cfg(test)]
            fail_next_profile_update: false,
            #[cfg(test)]
            fail_next_remove: false,
        })
    }

    pub fn data_root(&self) -> &Path {
        &self.data_root
    }

    /// Acquires the character's non-blocking, process-owned advisory lock.
    pub fn acquire_lock(&self, id: &CharacterId) -> Result<CharacterLock, StorageError> {
        let lock_directory = self.data_root.join("locks");
        fs::create_dir_all(&lock_directory)?;
        restrict_permissions(&lock_directory, 0o700)?;
        let lock_path = lock_directory.join(format!("{id}.lock"));
        let mut file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .truncate(false)
            .open(&lock_path)?;
        restrict_permissions(&lock_path, 0o600)?;
        match file.try_lock_exclusive() {
            Ok(()) => {
                file.set_len(0)?;
                file.write_all(format!("{}\n", std::process::id()).as_bytes())?;
                file.sync_data()?;
                Ok(CharacterLock { _file: file })
            }
            Err(error) if error.kind() == ErrorKind::WouldBlock => {
                Err(StorageError::AlreadyOwned(id.clone()))
            }
            Err(error) => Err(StorageError::Io(error)),
        }
    }

    pub(crate) fn acquire_online_action_lock(
        &self,
        id: &CharacterId,
    ) -> Result<OnlineActionLock, StorageError> {
        let lock_directory = self.data_root.join("locks");
        fs::create_dir_all(&lock_directory)?;
        restrict_permissions(&lock_directory, 0o700)?;
        let lock_path = lock_directory.join(format!("{id}.online.lock"));
        let file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .truncate(false)
            .open(&lock_path)?;
        restrict_permissions(&lock_path, 0o600)?;
        file.lock_exclusive()?;
        Ok(OnlineActionLock { _file: file })
    }

    pub fn is_owned(&self, id: &CharacterId) -> Result<bool, StorageError> {
        match self.acquire_lock(id) {
            Ok(lock) => {
                drop(lock);
                Ok(false)
            }
            Err(StorageError::AlreadyOwned(_)) => Ok(true),
            Err(error) => Err(error),
        }
    }

    /// Returns the process identifier recorded by the current lock owner.
    pub fn owner_process_id(&self, id: &CharacterId) -> Result<Option<u32>, StorageError> {
        let lock_path = self.data_root.join("locks").join(format!("{id}.lock"));
        match fs::read_to_string(lock_path) {
            Ok(value) => Ok(value.trim().parse().ok()),
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
            Err(error) => Err(StorageError::Io(error)),
        }
    }

    pub fn register(&mut self, character: &Character) -> Result<ManagedCharacter, StorageError> {
        let canonical_state = canonical_json(character)?;
        let original_document =
            serde_json::to_string(&character.document).map_err(StorageError::StateJson)?;
        let id = CharacterId::new();
        let identity = identity(character);
        let now = unix_millis();

        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        transaction.execute(
            "INSERT INTO characters (
                id, name, race, character_class, level, canonical_state,
                canonical_state_version, original_document, motto, guild,
                created_at_unix_ms, updated_at_unix_ms
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                id.to_string(),
                identity.name,
                identity.race,
                identity.class,
                sqlite_integer(identity.level, "level")?,
                canonical_state,
                CANONICAL_STATE_VERSION,
                original_document,
                character.profile.motto,
                character.profile.guild,
                now,
                now,
            ],
        )?;
        transaction.commit()?;

        Ok(ManagedCharacter {
            id,
            identity,
            state_version: CANONICAL_STATE_VERSION,
            created_at_unix_ms: now,
            updated_at_unix_ms: now,
            state: character.clone(),
        })
    }

    pub fn list(&self) -> Result<Vec<ManagedCharacter>, StorageError> {
        let mut statement = self.connection.prepare(
            "SELECT id, name, race, character_class, level, canonical_state,
                    canonical_state_version, motto, guild, created_at_unix_ms,
                    updated_at_unix_ms
             FROM characters ORDER BY created_at_unix_ms, id",
        )?;
        let mut rows = statement.query([])?;
        let mut characters = Vec::new();
        while let Some(row) = rows.next()? {
            characters.push(managed_character_from_row(row)?);
        }
        Ok(characters)
    }

    pub fn get(&self, id: &CharacterId) -> Result<ManagedCharacter, StorageError> {
        let mut statement = self.connection.prepare(
            "SELECT id, name, race, character_class, level, canonical_state,
                    canonical_state_version, motto, guild, created_at_unix_ms,
                    updated_at_unix_ms
             FROM characters WHERE id = ?1",
        )?;
        let mut rows = statement.query([id.to_string()])?;
        match rows.next()? {
            Some(row) => managed_character_from_row(row),
            None => Err(StorageError::NotFound(id.clone())),
        }
    }

    pub fn profile(&self, id: &CharacterId) -> Result<OnlineProfile, StorageError> {
        self.connection
            .query_row(
                "SELECT motto, guild FROM characters WHERE id = ?1",
                [id.to_string()],
                |row| {
                    Ok(OnlineProfile {
                        motto: row.get(0)?,
                        guild: row.get(1)?,
                    })
                },
            )
            .optional()?
            .ok_or_else(|| StorageError::NotFound(id.clone()))
    }

    pub fn replace_profile(
        &mut self,
        id: &CharacterId,
        profile: &OnlineProfile,
    ) -> Result<(), StorageError> {
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;

        #[cfg(test)]
        if std::mem::take(&mut self.fail_next_profile_update) {
            transaction.execute(
                "UPDATE characters SET motto = ?1 WHERE id = ?2",
                params![profile.motto, id.to_string()],
            )?;
            return Err(StorageError::InjectedFailure);
        }

        let changed = transaction.execute(
            "UPDATE characters
             SET motto = ?1, guild = ?2, updated_at_unix_ms = ?3
             WHERE id = ?4",
            params![profile.motto, profile.guild, unix_millis(), id.to_string()],
        )?;
        if changed == 0 {
            return Err(StorageError::NotFound(id.clone()));
        }
        transaction.commit()?;
        Ok(())
    }

    /// Resolves an online credential for the active worker that already owns
    /// this character.
    pub(crate) fn reporting_target_for_worker(
        &self,
        id: &CharacterId,
    ) -> Result<ReportingTarget, StorageError> {
        let online_action_lock = self.acquire_online_action_lock(id)?;
        let character = self.get(id)?;
        if character.state.online.is_none() {
            return Err(StorageError::ReportingIneligible);
        }
        let source = self
            .connection
            .query_row(
                "SELECT original_document FROM characters WHERE id = ?1",
                [id.to_string()],
                |row| row.get::<_, String>(0),
            )
            .optional()?
            .ok_or_else(|| StorageError::NotFound(id.clone()))?;
        let document: serde_json::Value =
            serde_json::from_str(&source).map_err(StorageError::StateJson)?;
        let passkey = document["online"]["passkey"]
            .as_i64()
            .and_then(|value| i32::try_from(value).ok())
            .ok_or(StorageError::ReportingIneligible)?;
        Ok(ReportingTarget {
            identity: character.identity,
            state: character.state,
            passkey,
            _online_action_lock: online_action_lock,
        })
    }

    /// Resolves an online credential while holding the short-lived online
    /// action lock, allowing explicit online actions during active simulation.
    pub(crate) fn online_action_target(
        &self,
        id: &CharacterId,
    ) -> Result<ReportingTarget, StorageError> {
        let online_action_lock = self.acquire_online_action_lock(id)?;
        let character = self.get(id)?;
        if character.state.online.is_none() {
            return Err(StorageError::ReportingIneligible);
        }
        let source = self
            .connection
            .query_row(
                "SELECT original_document FROM characters WHERE id = ?1",
                [id.to_string()],
                |row| row.get::<_, String>(0),
            )
            .optional()?
            .ok_or_else(|| StorageError::NotFound(id.clone()))?;
        let document: serde_json::Value =
            serde_json::from_str(&source).map_err(StorageError::StateJson)?;
        let passkey = document["online"]["passkey"]
            .as_i64()
            .and_then(|value| i32::try_from(value).ok())
            .ok_or(StorageError::ReportingIneligible)?;
        Ok(ReportingTarget {
            identity: character.identity,
            state: character.state,
            passkey,
            _online_action_lock: online_action_lock,
        })
    }

    /// Atomically removes every persisted record for an inactive character.
    ///
    /// Holding the advisory lock while the transaction runs prevents a worker
    /// from acquiring ownership between the deletion check and mutation.
    pub fn remove(&mut self, id: &CharacterId) -> Result<(), StorageError> {
        let lock = self.acquire_lock(id)?;
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;

        #[cfg(test)]
        if std::mem::take(&mut self.fail_next_remove) {
            transaction.execute("DELETE FROM characters WHERE id = ?1", [id.to_string()])?;
            return Err(StorageError::InjectedFailure);
        }

        let changed =
            transaction.execute("DELETE FROM characters WHERE id = ?1", [id.to_string()])?;
        if changed == 0 {
            return Err(StorageError::NotFound(id.clone()));
        }
        transaction.commit()?;
        drop(lock);
        Ok(())
    }

    /// Atomically records a successful simulation result.
    pub fn replace_state(
        &mut self,
        id: &CharacterId,
        character: &Character,
    ) -> Result<(), StorageError> {
        let canonical_state = canonical_json(character)?;
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;

        #[cfg(test)]
        if std::mem::take(&mut self.fail_next_update) {
            transaction.execute(
                "UPDATE characters SET updated_at_unix_ms = ?1 WHERE id = ?2",
                params![unix_millis(), id.to_string()],
            )?;
            return Err(StorageError::InjectedFailure);
        }

        let changed = transaction.execute(
            "UPDATE characters
             SET name = ?1, race = ?2, character_class = ?3, level = ?4,
                 canonical_state = ?5, canonical_state_version = ?6,
                 updated_at_unix_ms = ?7
             WHERE id = ?8",
            params![
                character.traits.name,
                character.traits.race,
                character.traits.class,
                sqlite_integer(character.traits.level, "level")?,
                canonical_state,
                CANONICAL_STATE_VERSION,
                unix_millis(),
                id.to_string()
            ],
        )?;
        if changed == 0 {
            return Err(StorageError::NotFound(id.clone()));
        }
        transaction.commit()?;
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn original_document(&self, id: &CharacterId) -> Result<Value, StorageError> {
        let source = self
            .connection
            .query_row(
                "SELECT original_document FROM characters WHERE id = ?1",
                [id.to_string()],
                |row| row.get::<_, String>(0),
            )
            .optional()?
            .ok_or_else(|| StorageError::NotFound(id.clone()))?;
        serde_json::from_str(&source).map_err(StorageError::StateJson)
    }

    #[cfg(test)]
    fn inject_next_update_failure(&mut self) {
        self.fail_next_update = true;
    }

    #[cfg(test)]
    fn inject_next_profile_update_failure(&mut self) {
        self.fail_next_profile_update = true;
    }

    #[cfg(test)]
    fn inject_next_remove_failure(&mut self) {
        self.fail_next_remove = true;
    }
}

fn initialize_schema(connection: &mut Connection) -> Result<(), StorageError> {
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let version: i64 = transaction.pragma_query_value(None, "user_version", |row| row.get(0))?;
    if version > DATABASE_SCHEMA_VERSION {
        return Err(StorageError::UnsupportedSchema(version));
    }
    if version == DATABASE_SCHEMA_VERSION {
        transaction.commit()?;
        return Ok(());
    }
    if version == 0 {
        transaction.execute_batch(
            "CREATE TABLE characters (
                id TEXT PRIMARY KEY NOT NULL,
                name TEXT NOT NULL,
                race TEXT NOT NULL,
                character_class TEXT NOT NULL,
                level INTEGER NOT NULL,
                canonical_state TEXT NOT NULL,
                canonical_state_version INTEGER NOT NULL,
                original_document TEXT NOT NULL,
                motto TEXT NOT NULL DEFAULT '',
                guild TEXT NOT NULL DEFAULT '',
                created_at_unix_ms INTEGER NOT NULL,
                updated_at_unix_ms INTEGER NOT NULL
            );",
        )?;
    } else if version == 1 {
        transaction.execute_batch(
            "ALTER TABLE characters ADD COLUMN motto TEXT NOT NULL DEFAULT '';
             ALTER TABLE characters ADD COLUMN guild TEXT NOT NULL DEFAULT '';",
        )?;
        let profiles = {
            let mut statement =
                transaction.prepare("SELECT id, original_document FROM characters")?;
            let rows = statement.query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })?;
            rows.collect::<Result<Vec<_>, _>>()?
        };
        for (id, source) in profiles {
            let document = serde_json::from_str::<serde_json::Value>(&source).ok();
            let motto = document
                .as_ref()
                .and_then(|value| value.get("motto"))
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default();
            let guild = document
                .as_ref()
                .and_then(|value| value.get("guild"))
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default();
            transaction.execute(
                "UPDATE characters SET motto = ?1, guild = ?2 WHERE id = ?3",
                params![motto, guild, id],
            )?;
        }
    }
    transaction.pragma_update(None, "user_version", DATABASE_SCHEMA_VERSION)?;
    transaction.commit()?;
    Ok(())
}

fn restrict_permissions(path: &Path, mode: u32) -> Result<(), StorageError> {
    fs::set_permissions(path, fs::Permissions::from_mode(mode))?;
    Ok(())
}

fn managed_character_from_row(row: &rusqlite::Row<'_>) -> Result<ManagedCharacter, StorageError> {
    let id = CharacterId::parse(&row.get::<_, String>(0)?)?;
    let mut state: Character =
        serde_json::from_str(&row.get::<_, String>(5)?).map_err(StorageError::StateJson)?;
    state.profile = OnlineProfile {
        motto: row.get(7)?,
        guild: row.get(8)?,
    };
    Ok(ManagedCharacter {
        id,
        identity: CharacterIdentity {
            name: row.get(1)?,
            race: row.get(2)?,
            class: row.get(3)?,
            level: u64::try_from(row.get::<_, i64>(4)?)
                .map_err(|_| StorageError::IntegerOutOfRange("level"))?,
        },
        state_version: row.get(6)?,
        created_at_unix_ms: row.get(9)?,
        updated_at_unix_ms: row.get(10)?,
        state,
    })
}

fn canonical_json(character: &Character) -> Result<String, StorageError> {
    let mut canonical = character.clone();
    canonical.profile = OnlineProfile::default();
    serde_json::to_string(&canonical).map_err(StorageError::StateJson)
}

fn identity(character: &Character) -> CharacterIdentity {
    CharacterIdentity {
        name: character.traits.name.clone(),
        race: character.traits.race.clone(),
        class: character.traits.class.clone(),
        level: character.traits.level,
    }
}

fn unix_millis() -> i64 {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    millis.min(i64::MAX as u128) as i64
}

fn sqlite_integer(value: u64, name: &'static str) -> Result<i64, StorageError> {
    i64::try_from(value).map_err(|_| StorageError::IntegerOutOfRange(name))
}

#[derive(Debug, Error)]
pub enum WorkerError {
    #[error(transparent)]
    Storage(#[from] StorageError),
    #[error(transparent)]
    Simulation(#[from] crate::simulation::SimulationError),
    #[error("worker interval must be greater than zero")]
    ZeroInterval,
    #[error("elapsed worker duration is too large to represent in milliseconds")]
    ElapsedOverflow,
}

/// Selects the elapsed time contributed by one active worker callback.
///
/// Scheduler delay beyond `interval` is intentionally discarded.  The caller
/// owns resetting its timing baseline after every callback attempt.
pub fn select_worker_elapsed(previous: Instant, current: Instant, interval: Duration) -> Duration {
    current
        .checked_duration_since(previous)
        .unwrap_or(Duration::ZERO)
        .min(interval)
}

/// Returns the tick-aligned simulated duration until the active task completes.
pub fn aligned_task_completion_duration(state: &Character) -> Duration {
    let remaining_ms = (state.progress.task.max as f64 - state.progress.task.position)
        .max(0.0)
        .ceil() as u64;
    let ticks = remaining_ms.max(1).div_ceil(crate::simulation::MAX_TICK_MS);
    Duration::from_millis(ticks.saturating_mul(crate::simulation::MAX_TICK_MS))
}

/// Owns one managed character and advances it only while this process is active.
pub struct Worker {
    store: Store,
    id: CharacterId,
    _lock: CharacterLock,
    last_tick: Instant,
    transport: Box<dyn ReportTransport>,
}

impl Worker {
    /// Takes ownership before reading any persisted character state.
    pub fn start(store: Store, id: CharacterId) -> Result<Self, WorkerError> {
        let lock = store.acquire_lock(&id)?;
        store.get(&id)?;
        Ok(Self {
            store,
            id,
            _lock: lock,
            last_tick: Instant::now(),
            transport: Box::new(HttpsTransport),
        })
    }

    #[cfg(test)]
    fn start_with_transport(
        store: Store,
        id: CharacterId,
        transport: impl ReportTransport + 'static,
    ) -> Result<Self, WorkerError> {
        let mut worker = Self::start(store, id)?;
        worker.transport = Box::new(transport);
        Ok(worker)
    }

    /// Advances with an explicit duration. This is useful for deterministic
    /// embedders and tests; the long-running worker uses [`Self::run_until`].
    pub fn advance_elapsed(&mut self, elapsed: Duration) -> Result<(), WorkerError> {
        let elapsed_ms: u64 = elapsed
            .as_millis()
            .try_into()
            .map_err(|_| WorkerError::ElapsedOverflow)?;
        if elapsed_ms == 0 {
            return Ok(());
        }
        let state = self.store.get(&self.id)?.state;
        let trace = crate::simulation::advance_with_trace(
            &state,
            &crate::ruleset::BUNDLED,
            elapsed_ms,
            "",
        )?;
        self.store.replace_state(&self.id, &trace.state)?;
        for event in &trace.events {
            let _ = crate::reporting::submit_event(&self.store, &self.id, event, &*self.transport);
        }
        Ok(())
    }

    /// Runs periodic updates from a monotonic clock until `stop` is requested.
    ///
    /// The baseline is created when the worker starts, not from persisted wall
    /// time, so a later worker never applies downtime as game time.
    pub fn run_until(mut self, stop: &AtomicBool, interval: Duration) -> Result<(), WorkerError> {
        if interval.is_zero() {
            return Err(WorkerError::ZeroInterval);
        }
        while !stop.load(Ordering::Relaxed) {
            let state = self.store.get(&self.id)?.state;
            let scheduled = aligned_task_completion_duration(&state).min(interval);
            thread::sleep(scheduled);
            if stop.load(Ordering::Relaxed) {
                break;
            }
            let now = Instant::now();
            let elapsed = select_worker_elapsed(self.last_tick, now, scheduled);
            self.last_tick = now;
            self.advance_elapsed(elapsed)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::{Path, PathBuf},
        thread,
        time::Duration,
    };

    use base64::{Engine, engine::general_purpose::STANDARD};
    use serde_json::json;
    use std::sync::{Arc, Mutex, mpsc};
    use url::Url;

    use super::*;
    use crate::{
        checkpoint,
        reporting::{DeliveryOutcome, OFFICIAL_LEADERBOARD_ENDPOINT, ReportTransport},
        save,
    };

    struct TestDirectory(PathBuf);

    impl TestDirectory {
        fn new(label: &str) -> Self {
            let path = Path::new("target")
                .join("gyrognome-runtime-tests")
                .join(format!("{label}-{}", Uuid::new_v4()));
            fs::create_dir_all(&path).unwrap();
            Self(path)
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn fixture_character() -> Character {
        save::import_text(&STANDARD.encode(include_str!("../tests/fixtures/reference-save.json")))
            .unwrap()
    }

    fn create_version_one_database(
        data_root: &Path,
        character: &Character,
    ) -> (CharacterId, String, String) {
        fs::create_dir_all(data_root).unwrap();
        let connection = Connection::open(data_root.join(DATABASE_FILENAME)).unwrap();
        connection
            .execute_batch(
                "CREATE TABLE characters (
                    id TEXT PRIMARY KEY NOT NULL,
                    name TEXT NOT NULL,
                    race TEXT NOT NULL,
                    character_class TEXT NOT NULL,
                    level INTEGER NOT NULL,
                    canonical_state TEXT NOT NULL,
                    canonical_state_version INTEGER NOT NULL,
                    original_document TEXT NOT NULL,
                    created_at_unix_ms INTEGER NOT NULL,
                    updated_at_unix_ms INTEGER NOT NULL
                );
                PRAGMA user_version = 1;",
            )
            .unwrap();
        let id = CharacterId::new();
        let canonical_state = serde_json::to_string(character).unwrap();
        let original_document = serde_json::to_string(&character.document).unwrap();
        connection
            .execute(
                "INSERT INTO characters (
                    id, name, race, character_class, level, canonical_state,
                    canonical_state_version, original_document, created_at_unix_ms,
                    updated_at_unix_ms
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    id.to_string(),
                    character.traits.name,
                    character.traits.race,
                    character.traits.class,
                    sqlite_integer(character.traits.level, "level").unwrap(),
                    canonical_state,
                    CANONICAL_STATE_VERSION,
                    original_document,
                    1_i64,
                    2_i64,
                ],
            )
            .unwrap();
        (id, canonical_state, original_document)
    }

    #[test]
    fn resolves_xdg_data_home_and_initializes_a_database() {
        let directory = TestDirectory::new("xdg");
        assert_eq!(
            data_root_from(Some(&directory.0), None).unwrap(),
            directory.0.join("gyrognome")
        );
        assert_eq!(
            data_root_from(None, Some(&directory.0)).unwrap(),
            directory.0.join(".local/share/gyrognome")
        );

        let store = Store::open_at(directory.0.join("gyrognome")).unwrap();
        assert!(store.data_root().join(DATABASE_FILENAME).is_file());
        assert!(store.list().unwrap().is_empty());
        assert_eq!(
            fs::metadata(store.data_root())
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o700
        );
        assert_eq!(
            fs::metadata(store.data_root().join(DATABASE_FILENAME))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
    }

    #[test]
    fn migrates_version_one_profile_values_without_changing_private_data() {
        let directory = TestDirectory::new("profile-migration");
        let mut character = fixture_character();
        character.document["motto"] = json!("Steady progress");
        character.document["guild"] = json!("Gnomes");
        let (id, canonical_state, original_document) =
            create_version_one_database(&directory.0, &character);

        let store = Store::open_at(&directory.0).unwrap();

        let version: i64 = store
            .connection
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .unwrap();
        assert_eq!(version, DATABASE_SCHEMA_VERSION);
        let persisted: (String, String, String, String) = store
            .connection
            .query_row(
                "SELECT motto, guild, canonical_state, original_document
                 FROM characters WHERE id = ?1",
                [id.to_string()],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .unwrap();
        assert_eq!(persisted.0, "Steady progress");
        assert_eq!(persisted.1, "Gnomes");
        assert_eq!(persisted.2, canonical_state);
        assert_eq!(persisted.3, original_document);
        assert_eq!(
            serde_json::from_str::<Value>(&persisted.3).unwrap()["online"]["passkey"],
            4242
        );
    }

    #[test]
    fn migration_defaults_missing_invalid_or_malformed_profile_values() {
        for (label, document) in [
            ("missing", json!({"online": {"passkey": 4242}})),
            ("invalid", json!({"motto": 7, "guild": false})),
            ("malformed", Value::String("not json".to_owned())),
        ] {
            let directory = TestDirectory::new(label);
            let mut character = fixture_character();
            character.document = document;
            let (id, _, _) = create_version_one_database(&directory.0, &character);
            if label == "malformed" {
                Store::open_at(&directory.0)
                    .unwrap()
                    .connection
                    .execute(
                        "UPDATE characters SET original_document = 'not json' WHERE id = ?1",
                        [id.to_string()],
                    )
                    .unwrap();
            }

            let store = Store::open_at(&directory.0).unwrap();
            let profile: (String, String) = store
                .connection
                .query_row(
                    "SELECT motto, guild FROM characters WHERE id = ?1",
                    [id.to_string()],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )
                .unwrap();
            assert_eq!(profile, (String::new(), String::new()));
        }
    }

    #[test]
    fn rejects_database_schemas_newer_than_supported() {
        let directory = TestDirectory::new("newer-schema");
        fs::create_dir_all(&directory.0).unwrap();
        let connection = Connection::open(directory.0.join(DATABASE_FILENAME)).unwrap();
        connection
            .pragma_update(None, "user_version", DATABASE_SCHEMA_VERSION + 1)
            .unwrap();
        drop(connection);

        assert!(matches!(
            Store::open_at(&directory.0),
            Err(StorageError::UnsupportedSchema(version))
                if version == DATABASE_SCHEMA_VERSION + 1
        ));
    }

    #[test]
    fn registers_safe_identity_and_preserves_original_document() {
        let directory = TestDirectory::new("register");
        let mut store = Store::open_at(&directory.0).unwrap();
        let character = fixture_character();
        let registered = store.register(&character).unwrap();

        assert_eq!(
            CharacterId::parse(&registered.id.to_string()).unwrap(),
            registered.id
        );
        assert_eq!(registered.identity.name, "Reference Hero");
        assert_eq!(
            store.original_document(&registered.id).unwrap(),
            character.document
        );
        let looked_up = store.get(&registered.id).unwrap();
        assert_eq!(looked_up.state.traits.name, "Reference Hero");
        assert_eq!(looked_up.state.document, Value::Null);
    }

    #[test]
    fn online_action_target_allows_active_or_inactive_online_credentials() {
        let directory = TestDirectory::new("reporting-target");
        let mut store = Store::open_at(&directory.0).unwrap();
        let registered = store.register(&fixture_character()).unwrap();
        let target = store.online_action_target(&registered.id).unwrap();
        assert_eq!(target.identity, registered.identity);
        assert_eq!(target.state.document, Value::Null);
        drop(target);

        let mut offline = fixture_character();
        offline.online = None;
        let offline = store.register(&offline).unwrap();
        assert!(matches!(
            store.online_action_target(&offline.id),
            Err(StorageError::ReportingIneligible)
        ));

        assert!(matches!(
            store.online_action_target(&CharacterId::new()),
            Err(StorageError::NotFound(_))
        ));

        let owned = store.register(&fixture_character()).unwrap();
        let worker =
            Worker::start(Store::open_at(&directory.0).unwrap(), owned.id.clone()).unwrap();
        let target = store.online_action_target(&owned.id).unwrap();
        assert_eq!(target.identity, owned.identity);
        assert!(store.is_owned(&owned.id).unwrap());
        drop(target);
        drop(worker);

        store
            .connection
            .execute(
                "UPDATE characters SET original_document = ?1 WHERE id = ?2",
                ["{}", registered.id.to_string().as_str()],
            )
            .unwrap();
        assert!(matches!(
            store.online_action_target(&registered.id),
            Err(StorageError::ReportingIneligible)
        ));

        store
            .connection
            .execute(
                "UPDATE characters SET original_document = ?1 WHERE id = ?2",
                ["not json", registered.id.to_string().as_str()],
            )
            .unwrap();
        assert!(matches!(
            store.online_action_target(&registered.id),
            Err(StorageError::StateJson(_))
        ));
    }

    #[test]
    fn invalid_import_creates_no_record() {
        let directory = TestDirectory::new("invalid-import");
        let store = Store::open_at(&directory.0).unwrap();
        assert!(save::import_text("definitely not a browser save").is_err());
        assert!(store.list().unwrap().is_empty());

        let malformed = json!({"Traits": {"Name": "missing everything"}});
        assert!(
            save::import_text(&STANDARD.encode(serde_json::to_vec(&malformed).unwrap())).is_err()
        );
        assert!(store.list().unwrap().is_empty());
    }

    #[test]
    fn failed_update_keeps_previous_state_after_reopen() {
        let directory = TestDirectory::new("transaction");
        let mut store = Store::open_at(&directory.0).unwrap();
        let registered = store.register(&fixture_character()).unwrap();
        let mut next = registered.state.clone();
        next.activity.tasks += 1;
        store.inject_next_update_failure();
        assert!(matches!(
            store.replace_state(&registered.id, &next),
            Err(StorageError::InjectedFailure)
        ));
        drop(store);

        let reopened = Store::open_at(&directory.0).unwrap();
        assert_eq!(
            reopened.get(&registered.id).unwrap().state.activity.tasks,
            registered.state.activity.tasks
        );
    }

    #[test]
    fn profile_updates_are_independent_from_canonical_state_writes() {
        let directory = TestDirectory::new("profile-state-independence");
        let mut state_store = Store::open_at(&directory.0).unwrap();
        let registered = state_store.register(&fixture_character()).unwrap();
        let mut stale_state = registered.state.clone();
        stale_state.activity.tasks += 1;

        let mut profile_store = Store::open_at(&directory.0).unwrap();
        let profile = OnlineProfile {
            motto: "Persist independently".to_owned(),
            guild: "Gnomes".to_owned(),
        };
        profile_store
            .replace_profile(&registered.id, &profile)
            .unwrap();
        state_store
            .replace_state(&registered.id, &stale_state)
            .unwrap();

        let reopened = Store::open_at(&directory.0).unwrap();
        let managed = reopened.get(&registered.id).unwrap();
        assert_eq!(managed.state.activity.tasks, stale_state.activity.tasks);
        assert_eq!(managed.state.profile, profile);
        assert_eq!(reopened.profile(&registered.id).unwrap(), profile);
    }

    #[test]
    fn failed_profile_update_retains_the_previous_complete_profile() {
        let directory = TestDirectory::new("profile-transaction");
        let mut store = Store::open_at(&directory.0).unwrap();
        let registered = store.register(&fixture_character()).unwrap();
        let original = OnlineProfile {
            motto: "Original motto".to_owned(),
            guild: "Original guild".to_owned(),
        };
        store.replace_profile(&registered.id, &original).unwrap();
        store.inject_next_profile_update_failure();

        assert!(matches!(
            store.replace_profile(
                &registered.id,
                &OnlineProfile {
                    motto: "Partial motto".to_owned(),
                    guild: "Replacement guild".to_owned(),
                }
            ),
            Err(StorageError::InjectedFailure)
        ));
        drop(store);

        let reopened = Store::open_at(&directory.0).unwrap();
        assert_eq!(reopened.profile(&registered.id).unwrap(), original);
        assert_eq!(
            reopened.get(&registered.id).unwrap().state.profile,
            original
        );
    }

    #[test]
    fn profile_operations_report_missing_characters() {
        let directory = TestDirectory::new("profile-missing");
        let mut store = Store::open_at(&directory.0).unwrap();
        let id = CharacterId::new();

        assert!(matches!(
            store.profile(&id),
            Err(StorageError::NotFound(missing)) if missing == id
        ));
        assert!(matches!(
            store.replace_profile(&id, &OnlineProfile::default()),
            Err(StorageError::NotFound(missing)) if missing == id
        ));
    }

    #[test]
    fn removes_all_persisted_character_data_atomically() {
        let directory = TestDirectory::new("remove");
        let mut store = Store::open_at(&directory.0).unwrap();
        let registered = store.register(&fixture_character()).unwrap();
        assert!(store.original_document(&registered.id).is_ok());

        store.remove(&registered.id).unwrap();

        assert!(matches!(
            store.get(&registered.id),
            Err(StorageError::NotFound(id)) if id == registered.id
        ));
        assert!(matches!(
            store.original_document(&registered.id),
            Err(StorageError::NotFound(id)) if id == registered.id
        ));
    }

    #[test]
    fn failed_remove_keeps_the_complete_character_after_reopen() {
        let directory = TestDirectory::new("remove-transaction");
        let mut store = Store::open_at(&directory.0).unwrap();
        let registered = store.register(&fixture_character()).unwrap();
        store.inject_next_remove_failure();

        assert!(matches!(
            store.remove(&registered.id),
            Err(StorageError::InjectedFailure)
        ));
        drop(store);

        let reopened = Store::open_at(&directory.0).unwrap();
        assert_eq!(
            reopened.get(&registered.id).unwrap().identity,
            registered.identity
        );
        assert!(reopened.original_document(&registered.id).is_ok());
    }

    #[test]
    fn replaces_safe_identity_with_the_persisted_canonical_state() {
        let directory = TestDirectory::new("identity-update");
        let mut store = Store::open_at(&directory.0).unwrap();
        let registered = store.register(&fixture_character()).unwrap();
        let mut advanced = registered.state.clone();
        advanced.traits.level += 1;
        store.replace_state(&registered.id, &advanced).unwrap();

        assert_eq!(
            store.get(&registered.id).unwrap().identity.level,
            advanced.traits.level
        );
    }

    #[test]
    fn concurrent_store_initialization_is_idempotent() {
        let directory = TestDirectory::new("concurrent-initialization");
        let first_path = directory.0.clone();
        let second_path = directory.0.clone();
        let first = thread::spawn(move || Store::open_at(first_path).unwrap());
        let second = thread::spawn(move || Store::open_at(second_path).unwrap());

        first.join().unwrap();
        second.join().unwrap();
        assert!(
            Store::open_at(&directory.0)
                .unwrap()
                .list()
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn worker_owns_one_character_until_it_exits() {
        let directory = TestDirectory::new("lock");
        let mut store = Store::open_at(&directory.0).unwrap();
        let registered = store.register(&fixture_character()).unwrap();
        let first =
            Worker::start(Store::open_at(&directory.0).unwrap(), registered.id.clone()).unwrap();

        assert!(matches!(
            Worker::start(Store::open_at(&directory.0).unwrap(), registered.id.clone()),
            Err(WorkerError::Storage(StorageError::AlreadyOwned(id))) if id == registered.id
        ));
        assert!(
            Store::open_at(&directory.0)
                .unwrap()
                .is_owned(&registered.id)
                .unwrap()
        );

        drop(first);
        Worker::start(Store::open_at(&directory.0).unwrap(), registered.id).unwrap();
    }

    #[test]
    fn online_action_lock_serializes_without_using_worker_ownership() {
        let directory = TestDirectory::new("online-action-lock");
        let mut store = Store::open_at(&directory.0).unwrap();
        let registered = store.register(&fixture_character()).unwrap();
        let first = store.acquire_online_action_lock(&registered.id).unwrap();
        let path = directory.0.clone();
        let id = registered.id.clone();
        let (sender, receiver) = mpsc::channel();

        let waiting = thread::spawn(move || {
            let store = Store::open_at(path).unwrap();
            let second = store.acquire_online_action_lock(&id).unwrap();
            sender.send(()).unwrap();
            drop(second);
        });

        assert!(receiver.recv_timeout(Duration::from_millis(100)).is_err());
        drop(first);
        receiver.recv_timeout(Duration::from_secs(2)).unwrap();
        waiting.join().unwrap();
    }

    #[test]
    fn online_action_lock_does_not_block_worker_ownership_or_profile_updates() {
        let directory = TestDirectory::new("online-action-boundaries");
        let mut store = Store::open_at(&directory.0).unwrap();
        let registered = store.register(&fixture_character()).unwrap();
        let worker =
            Worker::start(Store::open_at(&directory.0).unwrap(), registered.id.clone()).unwrap();
        let online_action = store.acquire_online_action_lock(&registered.id).unwrap();

        let profile = OnlineProfile {
            motto: "Still running".to_owned(),
            guild: "Gnomes".to_owned(),
        };
        Store::open_at(&directory.0)
            .unwrap()
            .replace_profile(&registered.id, &profile)
            .unwrap();

        assert!(store.is_owned(&registered.id).unwrap());
        assert_eq!(store.profile(&registered.id).unwrap(), profile);
        drop(online_action);
        drop(worker);
    }

    #[test]
    fn refuses_to_remove_a_character_owned_by_a_worker() {
        let directory = TestDirectory::new("remove-owned");
        let mut store = Store::open_at(&directory.0).unwrap();
        let registered = store.register(&fixture_character()).unwrap();
        let worker =
            Worker::start(Store::open_at(&directory.0).unwrap(), registered.id.clone()).unwrap();

        assert!(matches!(
            store.remove(&registered.id),
            Err(StorageError::AlreadyOwned(id)) if id == registered.id
        ));
        assert!(store.get(&registered.id).is_ok());
        drop(worker);
    }

    #[test]
    fn worker_persists_active_progress_without_downtime_catchup() {
        let directory = TestDirectory::new("worker-progress");
        let mut store = Store::open_at(&directory.0).unwrap();
        let mut initial = fixture_character();
        initial.stats.best = "STR".to_owned();
        initial.beststat = "STR 1".to_owned();
        let registered = store.register(&initial).unwrap();
        let original_position = registered.state.progress.task.position;
        let mut worker =
            Worker::start(Store::open_at(&directory.0).unwrap(), registered.id.clone()).unwrap();
        worker.advance_elapsed(Duration::from_millis(25)).unwrap();
        drop(worker);

        let after_active = Store::open_at(&directory.0)
            .unwrap()
            .get(&registered.id)
            .unwrap();
        assert_eq!(
            after_active.state.progress.task.position,
            original_position + 25.0
        );
        assert_eq!(after_active.state.stats.best, "CHA");
        assert_eq!(after_active.state.beststat, "CHA 15");

        thread::sleep(Duration::from_millis(5));
        let mut restarted =
            Worker::start(Store::open_at(&directory.0).unwrap(), registered.id.clone()).unwrap();
        restarted.advance_elapsed(Duration::ZERO).unwrap();
        drop(restarted);
        assert_eq!(
            Store::open_at(&directory.0)
                .unwrap()
                .get(&registered.id)
                .unwrap()
                .state
                .progress
                .task
                .position,
            original_position + 25.0
        );
    }

    #[test]
    fn selected_worker_elapsed_is_bounded_and_clock_independent() {
        let previous = Instant::now();
        let interval = Duration::from_millis(100);

        assert_eq!(
            select_worker_elapsed(previous, previous.checked_add(interval).unwrap(), interval),
            interval
        );
        assert_eq!(
            select_worker_elapsed(
                previous,
                previous.checked_add(Duration::from_millis(25)).unwrap(),
                interval
            ),
            Duration::from_millis(25)
        );
        assert_eq!(
            select_worker_elapsed(
                previous,
                previous.checked_add(Duration::from_millis(250)).unwrap(),
                interval
            ),
            interval
        );
        assert_eq!(
            select_worker_elapsed(previous, previous, interval),
            Duration::ZERO
        );
    }

    #[test]
    fn task_completion_duration_is_tick_aligned_and_never_zero() {
        let mut state = fixture_character();
        state.progress.task.max = 1_000;

        for (position, expected) in [(800.0, 200), (801.0, 200), (1_000.0, 100), (1_100.0, 100)] {
            state.progress.task.position = position;
            let duration = aligned_task_completion_duration(&state);
            assert_eq!(duration, Duration::from_millis(expected));
            assert_eq!(
                duration.as_millis() % u128::from(crate::simulation::MAX_TICK_MS),
                0
            );
            assert!(duration >= Duration::from_millis(crate::simulation::MAX_TICK_MS));
        }

        state.progress.task.max = 0;
        state.progress.task.position = 0.0;
        assert_eq!(
            aligned_task_completion_duration(&state),
            Duration::from_millis(crate::simulation::MAX_TICK_MS)
        );
    }

    #[test]
    fn aligned_wakes_land_on_and_persist_the_task_boundary() {
        let directory = TestDirectory::new("aligned-boundary");
        let mut initial = fixture_character();
        initial.progress.task.max = 450;
        initial.progress.task.position = 0.0;
        initial.progress.task.percent = 0;
        let initial_tasks = initial.activity.tasks;
        let mut store = Store::open_at(&directory.0).unwrap();
        let registered = store.register(&initial).unwrap();
        let mut worker =
            Worker::start(Store::open_at(&directory.0).unwrap(), registered.id.clone()).unwrap();
        let interval = Duration::from_millis(200);

        for expected in [200.0, 400.0] {
            let state = worker.store.get(&registered.id).unwrap().state;
            let scheduled = aligned_task_completion_duration(&state).min(interval);
            worker.advance_elapsed(scheduled).unwrap();
            assert_eq!(
                worker
                    .store
                    .get(&registered.id)
                    .unwrap()
                    .state
                    .progress
                    .task
                    .position,
                expected
            );
        }

        let state = worker.store.get(&registered.id).unwrap().state;
        let boundary = aligned_task_completion_duration(&state).min(interval);
        assert_eq!(boundary, Duration::from_millis(100));
        worker.advance_elapsed(boundary).unwrap();

        let persisted = worker.store.get(&registered.id).unwrap().state;
        assert_eq!(persisted.activity.tasks, initial_tasks + 1);
        assert_ne!(persisted.activity.task, initial.activity.task);
        assert!(persisted.progress.task.position < persisted.progress.task.max as f64);
    }

    #[test]
    fn aligned_and_fixed_interval_sequences_produce_identical_state() {
        let initial = fixture_character();
        let total = Duration::from_millis(3_000);
        let interval = Duration::from_millis(1_000);

        let fixed = crate::simulation::advance(
            &initial,
            &crate::ruleset::BUNDLED,
            total.as_millis() as u64,
        )
        .unwrap();

        let mut aligned = initial.clone();
        let mut remaining = total;
        while !remaining.is_zero() {
            let elapsed = aligned_task_completion_duration(&aligned)
                .min(interval)
                .min(remaining);
            aligned = crate::simulation::advance(
                &aligned,
                &crate::ruleset::BUNDLED,
                elapsed.as_millis() as u64,
            )
            .unwrap();
            remaining -= elapsed;
        }

        assert_eq!(
            serde_json::to_value(&aligned).unwrap(),
            serde_json::to_value(&fixed).unwrap()
        );
        assert_eq!(aligned.seed, fixed.seed);
    }

    #[test]
    fn delayed_callback_does_not_accumulate_discarded_time() {
        let previous = Instant::now();
        let interval = Duration::from_millis(100);
        let delayed = previous.checked_add(Duration::from_millis(250)).unwrap();
        let next = delayed.checked_add(interval).unwrap();

        assert_eq!(
            select_worker_elapsed(previous, delayed, interval),
            Duration::from_millis(100)
        );
        assert_eq!(
            select_worker_elapsed(delayed, next, interval),
            Duration::from_millis(100)
        );
    }

    #[test]
    fn worker_leaves_last_good_state_on_simulation_or_storage_failure() {
        let directory = TestDirectory::new("worker-failures");
        let mut unsupported = fixture_character();
        unsupported.progress.task.position = unsupported.progress.task.max as f64;
        unsupported.queue = vec!["unported|0".to_owned()];

        let mut store = Store::open_at(&directory.0).unwrap();
        let registered = store.register(&unsupported).unwrap();
        let mut worker =
            Worker::start(Store::open_at(&directory.0).unwrap(), registered.id.clone()).unwrap();
        assert!(matches!(
            worker.advance_elapsed(Duration::from_millis(1)),
            Err(WorkerError::Simulation(_))
        ));
        assert_eq!(
            Store::open_at(&directory.0)
                .unwrap()
                .get(&registered.id)
                .unwrap()
                .state
                .queue,
            unsupported.queue
        );
        drop(worker);

        let storage_registered = store.register(&fixture_character()).unwrap();
        let storage_original = storage_registered.state.clone();
        let mut failing_worker = Worker::start(
            Store::open_at(&directory.0).unwrap(),
            storage_registered.id.clone(),
        )
        .unwrap();
        failing_worker.store.inject_next_update_failure();
        assert!(matches!(
            failing_worker.advance_elapsed(Duration::from_millis(1)),
            Err(WorkerError::Storage(StorageError::InjectedFailure))
        ));
        drop(failing_worker);
        assert_eq!(
            Store::open_at(&directory.0)
                .unwrap()
                .get(&storage_registered.id)
                .unwrap()
                .state
                .activity
                .tasks,
            storage_original.activity.tasks
        );
    }

    #[test]
    fn worker_leaves_persisted_state_unchanged_when_duration_overflows() {
        let directory = TestDirectory::new("elapsed-overflow");
        let mut store = Store::open_at(&directory.0).unwrap();
        let registered = store.register(&fixture_character()).unwrap();
        let original = registered.state.clone();
        let mut worker =
            Worker::start(Store::open_at(&directory.0).unwrap(), registered.id.clone()).unwrap();

        assert!(matches!(
            worker.advance_elapsed(Duration::new(u64::MAX, 0)),
            Err(WorkerError::ElapsedOverflow)
        ));
        drop(worker);

        assert_eq!(
            Store::open_at(&directory.0)
                .unwrap()
                .get(&registered.id)
                .unwrap()
                .state
                .progress
                .task
                .position,
            original.progress.task.position
        );
    }

    struct RecordingTransport {
        triggers: Arc<Mutex<Vec<String>>>,
        mottos: Arc<Mutex<Vec<String>>>,
        outcome: DeliveryOutcome,
    }

    impl ReportTransport for RecordingTransport {
        fn deliver(&self, request: Url) -> DeliveryOutcome {
            assert_eq!(
                request.query_pairs().find(|(key, _)| key == "k").unwrap().1,
                "CHA 16"
            );
            self.triggers.lock().unwrap().push(
                request
                    .query_pairs()
                    .find(|(key, _)| key == "t")
                    .unwrap()
                    .1
                    .into_owned(),
            );
            self.mottos.lock().unwrap().push(
                request
                    .query_pairs()
                    .find(|(key, _)| key == "m")
                    .unwrap()
                    .1
                    .into_owned(),
            );
            self.outcome
        }
    }

    #[test]
    fn worker_persists_trace_before_delivering_each_online_event_once() {
        let directory = TestDirectory::new("worker-reports");
        let mut initial = checkpoint::load(Path::new("tests/fixtures/checkpoint-level-up.json"))
            .unwrap()
            .initial;
        initial.online = fixture_character().online;
        initial.online.as_mut().unwrap().host = format!("{OFFICIAL_LEADERBOARD_ENDPOINT}?");
        initial.document = fixture_character().document;
        initial.document["online"]["host"] =
            serde_json::Value::String(format!("{OFFICIAL_LEADERBOARD_ENDPOINT}?"));
        initial.queue = vec!["plot|1|Loading".to_owned()];
        initial.stats.best = "STR".to_owned();
        initial.beststat = "STR 1".to_owned();
        let mut store = Store::open_at(&directory.0).unwrap();
        let registered = store.register(&initial).unwrap();
        store
            .replace_profile(
                &registered.id,
                &OnlineProfile {
                    motto: "Worker profile motto".to_owned(),
                    guild: String::new(),
                },
            )
            .unwrap();
        let triggers = Arc::new(Mutex::new(Vec::new()));
        let mottos = Arc::new(Mutex::new(Vec::new()));
        let transport = RecordingTransport {
            triggers: Arc::clone(&triggers),
            mottos: Arc::clone(&mottos),
            outcome: DeliveryOutcome::Delivered,
        };
        let mut worker = Worker::start_with_transport(
            Store::open_at(&directory.0).unwrap(),
            registered.id.clone(),
            transport,
        )
        .unwrap();

        worker
            .advance_elapsed(Duration::from_millis(1_000))
            .unwrap();

        assert_eq!(*triggers.lock().unwrap(), ["l", "a"]);
        assert_eq!(
            *mottos.lock().unwrap(),
            [
                "Worker profile motto".to_owned(),
                "Worker profile motto".to_owned()
            ]
        );
        let persisted = Store::open_at(&directory.0)
            .unwrap()
            .get(&registered.id)
            .unwrap()
            .state;
        assert_eq!(persisted.plot.act, 1);
        assert_eq!(persisted.stats.best, "CHA");
        assert_eq!(persisted.beststat, "CHA 16");
    }

    #[test]
    fn worker_suppresses_offline_events_and_keeps_progress_on_delivery_failures() {
        let directory = TestDirectory::new("worker-report-failures");
        let mut online = checkpoint::load(Path::new("tests/fixtures/checkpoint-level-up.json"))
            .unwrap()
            .initial;
        online.online = fixture_character().online;
        online.online.as_mut().unwrap().host = format!("{OFFICIAL_LEADERBOARD_ENDPOINT}?");
        online.document = fixture_character().document;
        online.document["online"]["host"] =
            serde_json::Value::String(format!("{OFFICIAL_LEADERBOARD_ENDPOINT}?"));
        let mut store = Store::open_at(&directory.0).unwrap();
        let online_registered = store.register(&online).unwrap();
        let failed_triggers = Arc::new(Mutex::new(Vec::new()));
        let failed_mottos = Arc::new(Mutex::new(Vec::new()));
        let failed = RecordingTransport {
            triggers: Arc::clone(&failed_triggers),
            mottos: Arc::clone(&failed_mottos),
            outcome: DeliveryOutcome::DeliveryFailed,
        };
        let mut online_worker = Worker::start_with_transport(
            Store::open_at(&directory.0).unwrap(),
            online_registered.id.clone(),
            failed,
        )
        .unwrap();
        online_worker
            .advance_elapsed(Duration::from_millis(1_000))
            .unwrap();
        assert_eq!(*failed_triggers.lock().unwrap(), ["l"]);
        assert_eq!(
            Store::open_at(&directory.0)
                .unwrap()
                .get(&online_registered.id)
                .unwrap()
                .state
                .traits
                .level,
            2
        );

        let mut offline = online;
        offline.online = None;
        let offline_registered = store.register(&offline).unwrap();
        let suppressed_triggers = Arc::new(Mutex::new(Vec::new()));
        let suppressed_mottos = Arc::new(Mutex::new(Vec::new()));
        let suppressed = RecordingTransport {
            triggers: Arc::clone(&suppressed_triggers),
            mottos: Arc::clone(&suppressed_mottos),
            outcome: DeliveryOutcome::Delivered,
        };
        let mut offline_worker = Worker::start_with_transport(
            Store::open_at(&directory.0).unwrap(),
            offline_registered.id.clone(),
            suppressed,
        )
        .unwrap();
        offline_worker
            .advance_elapsed(Duration::from_millis(1_000))
            .unwrap();
        assert!(suppressed_triggers.lock().unwrap().is_empty());
    }
}
