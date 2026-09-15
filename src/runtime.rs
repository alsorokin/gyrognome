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

use crate::state::Character;
#[cfg(test)]
use rusqlite::OptionalExtension;
#[cfg(test)]
use serde_json::Value;

const DATABASE_FILENAME: &str = "characters.sqlite3";
const DATABASE_SCHEMA_VERSION: i64 = 1;
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
}

/// An advisory lock held for a managed character's worker lifetime.
pub struct CharacterLock {
    _file: File,
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
                canonical_state_version, original_document, created_at_unix_ms,
                updated_at_unix_ms
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                id.to_string(),
                identity.name,
                identity.race,
                identity.class,
                sqlite_integer(identity.level, "level")?,
                canonical_state,
                CANONICAL_STATE_VERSION,
                original_document,
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
                    canonical_state_version, created_at_unix_ms, updated_at_unix_ms
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
                    canonical_state_version, created_at_unix_ms, updated_at_unix_ms
             FROM characters WHERE id = ?1",
        )?;
        let mut rows = statement.query([id.to_string()])?;
        match rows.next()? {
            Some(row) => managed_character_from_row(row),
            None => Err(StorageError::NotFound(id.clone())),
        }
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
    transaction.execute_batch(
        "CREATE TABLE IF NOT EXISTS characters (
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
        );",
    )?;
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
    let state = serde_json::from_str(&row.get::<_, String>(5)?).map_err(StorageError::StateJson)?;
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
        created_at_unix_ms: row.get(7)?,
        updated_at_unix_ms: row.get(8)?,
        state,
    })
}

fn canonical_json(character: &Character) -> Result<String, StorageError> {
    serde_json::to_string(character).map_err(StorageError::StateJson)
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

/// Owns one managed character and advances it only while this process is active.
pub struct Worker {
    store: Store,
    id: CharacterId,
    _lock: CharacterLock,
    last_tick: Instant,
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
        })
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
        let next = crate::simulation::advance(&state, &crate::ruleset::BUNDLED, elapsed_ms)?;
        self.store.replace_state(&self.id, &next)?;
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
            thread::sleep(interval);
            if stop.load(Ordering::Relaxed) {
                break;
            }
            let now = Instant::now();
            self.advance_elapsed(now.duration_since(self.last_tick))?;
            self.last_tick = now;
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

    use super::*;
    use crate::save;

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
    fn worker_persists_active_progress_without_downtime_catchup() {
        let directory = TestDirectory::new("worker-progress");
        let mut store = Store::open_at(&directory.0).unwrap();
        let registered = store.register(&fixture_character()).unwrap();
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
}
