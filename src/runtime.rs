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
    compatibility::{
        CompatibilityProfile, CompatibilityState, DesktopAdvancementProvenance,
        DesktopCanonicalState, DesktopImportMetadata, ImportMetadata, RandomContinuation,
        SourceFormat, initialize_desktop_registration_random,
    },
    desktop_callback::{DesktopCallbackCheckpoint, DesktopCallbackObservation},
    desktop_eligibility::{
        DesktopEligibilityDecision, DesktopEligibilityInput, DesktopIneligibilityReason,
        DesktopOnlineOperation, DesktopOperationEligibility,
        evaluate_production_desktop_eligibility,
    },
    desktop_save::DesktopValidatedSave,
    desktop_simulation::SourceDerivedDesktopHooks,
    newguy::RandomSource,
    reporting::{HttpsTransport, ReportTransport},
    state::{Character, OnlineProfile},
};
use rusqlite::OptionalExtension;
#[cfg(test)]
use serde_json::Value;

const DATABASE_FILENAME: &str = "characters.sqlite3";
const DATABASE_SCHEMA_VERSION: i64 = 4;
pub const CANONICAL_STATE_VERSION: u32 = 2;

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

#[derive(Debug, Clone, Serialize)]
pub struct RegisteredDesktopCharacter {
    pub id: CharacterId,
    pub identity: CharacterIdentity,
    pub state_version: u32,
    pub created_at_unix_ms: i64,
    pub updated_at_unix_ms: i64,
    pub state: DesktopCanonicalState,
    pub compatibility: CompatibilityState,
    pub import_metadata: DesktopImportMetadata,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagedCompatibilityPresentation {
    pub profile: CompatibilityProfile,
    pub realm: Option<String>,
    pub online_eligibility: Vec<DesktopOperationEligibility>,
    pub unavailable_history: Option<crate::compatibility::DesktopUnavailableHistory>,
    pub measured_since_import: Option<crate::compatibility::SinceImportCounters>,
    pub advancement_provenance: Option<DesktopAdvancementProvenance>,
    pub notice: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagedCharacterSummary {
    pub id: CharacterId,
    pub identity: CharacterIdentity,
    pub updated_at_unix_ms: i64,
    pub compatibility: ManagedCompatibilityPresentation,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "stateProfile", content = "character", rename_all = "kebab-case")]
pub enum ManagedInspectionState {
    Browser(ManagedCharacter),
    Desktop644(RegisteredDesktopCharacter),
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagedInspection {
    pub compatibility: ManagedCompatibilityPresentation,
    pub state: ManagedInspectionState,
}

#[allow(dead_code)]
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct DesktopAuthentication {
    pub(crate) passkey: i32,
    pub(crate) realm: String,
    pub(crate) endpoint: String,
    pub(crate) account: String,
    pub(crate) password: String,
}

impl DesktopAuthentication {
    fn has_online_origin(&self) -> bool {
        self.passkey > 0
    }
}

fn desktop_progress_reporting_evidence_available(
    character: &RegisteredDesktopCharacter,
    authentication: &DesktopAuthentication,
) -> bool {
    [
        DesktopOnlineOperation::AutomaticLevel,
        DesktopOnlineOperation::AutomaticAct,
    ]
    .into_iter()
    .all(|operation| {
        desktop_eligibility_decision(character, authentication, &character.state, operation, None)
            == DesktopEligibilityDecision::Eligible
    })
}

fn should_mark_desktop_local_only(
    authentication: &DesktopAuthentication,
    changed: bool,
    progress_reporting_evidence_available: bool,
) -> bool {
    changed && authentication.has_online_origin() && !progress_reporting_evidence_available
}

fn ensure_desktop_online_eligible(
    character: &RegisteredDesktopCharacter,
    authentication: &DesktopAuthentication,
    request_state: &DesktopCanonicalState,
    operation: DesktopOnlineOperation,
    submitted_text: Option<&str>,
) -> Result<(), StorageError> {
    match desktop_eligibility_decision(
        character,
        authentication,
        request_state,
        operation,
        submitted_text,
    ) {
        DesktopEligibilityDecision::Eligible => Ok(()),
        DesktopEligibilityDecision::Ineligible(
            DesktopIneligibilityReason::FreshOfficialClientImportRequired,
        ) => Err(StorageError::FreshDesktopImportRequired),
        DesktopEligibilityDecision::Ineligible(reason) => {
            Err(StorageError::DesktopOnlineIneligible(reason))
        }
    }
}

fn desktop_eligibility_decision(
    character: &RegisteredDesktopCharacter,
    authentication: &DesktopAuthentication,
    request_state: &DesktopCanonicalState,
    operation: DesktopOnlineOperation,
    submitted_text: Option<&str>,
) -> DesktopEligibilityDecision {
    let input = DesktopEligibilityInput {
        profile: character.compatibility.profile,
        source_format: character.import_metadata.provenance.source_format,
        layout: character.import_metadata.provenance.recognized_layout,
        adaptations: &character.import_metadata.provenance.adaptations,
        advancement: character.import_metadata.advancement_provenance,
        passkey: authentication.passkey,
        realm: &authentication.realm,
        endpoint: &authentication.endpoint,
        account: &authentication.account,
        password: &authentication.password,
        encoding_supported: desktop_request_text_is_ascii(
            request_state,
            authentication,
            submitted_text,
        ),
        operation,
    };
    evaluate_production_desktop_eligibility(&input)
}

fn desktop_request_text_is_ascii(
    state: &DesktopCanonicalState,
    authentication: &DesktopAuthentication,
    submitted_text: Option<&str>,
) -> bool {
    crate::desktop_eligibility::desktop_request_text_is_ascii(
        state,
        &[
            &authentication.realm,
            &authentication.endpoint,
            &authentication.account,
            &authentication.password,
        ],
    ) && submitted_text.is_none_or(str::is_ascii)
}

impl std::fmt::Debug for DesktopAuthentication {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("DesktopAuthentication")
            .field("passkey", &"[redacted]")
            .field("realm", &"[redacted]")
            .field("endpoint", &"[redacted]")
            .field("account", &"[redacted]")
            .field("password", &"[redacted]")
            .finish()
    }
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
    #[error("desktop online operation is unavailable: {0}")]
    DesktopOnlineIneligible(DesktopIneligibilityReason),
    #[error(
        "desktop character advanced locally and requires a fresh official-client import for online use"
    )]
    FreshDesktopImportRequired,
    #[error("numeric value is outside SQLite's signed integer range: {0}")]
    IntegerOutOfRange(&'static str),
    #[error("desktop canonical state is invalid: {0}")]
    InvalidDesktopState(&'static str),
    #[error("persisted compatibility state is invalid: {0}")]
    InvalidCompatibilityState(&'static str),
    #[error("desktop since-import counter overflowed: {0}")]
    DesktopCounterOverflow(&'static str),
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
    #[cfg(test)]
    fail_next_desktop_registration: bool,
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

#[allow(dead_code)]
pub(crate) struct DesktopReportingTarget {
    pub(crate) identity: CharacterIdentity,
    pub(crate) state: DesktopCanonicalState,
    pub(crate) profile: OnlineProfile,
    pub(crate) authentication: DesktopAuthentication,
    pub(crate) adaptations: Vec<crate::compatibility::DesktopAdaptation>,
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
        initialize_schema(&mut connection, &data_root)?;
        restrict_store_files(&data_root)?;
        Ok(Self {
            data_root,
            connection,
            #[cfg(test)]
            fail_next_update: false,
            #[cfg(test)]
            fail_next_profile_update: false,
            #[cfg(test)]
            fail_next_remove: false,
            #[cfg(test)]
            fail_next_desktop_registration: false,
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
        let compatibility = CompatibilityState {
            profile: CompatibilityProfile::Browser,
            random: RandomContinuation::Browser(character.seed),
        };
        let random_continuation =
            serde_json::to_string(&compatibility.random).map_err(StorageError::StateJson)?;
        let import_metadata = serde_json::to_string(&ImportMetadata::Browser {
            source_format: SourceFormat::BrowserJson,
        })
        .map_err(StorageError::StateJson)?;
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
                compatibility_profile, random_continuation, import_metadata,
                created_at_unix_ms, updated_at_unix_ms
             ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15
             )",
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
                "browser",
                random_continuation,
                import_metadata,
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

    pub fn register_desktop(
        &mut self,
        save: &DesktopValidatedSave,
        random_source: &mut impl RandomSource,
    ) -> Result<RegisteredDesktopCharacter, StorageError> {
        let state = DesktopCanonicalState::from(save);
        let identity = desktop_identity(&state)?;
        let random = initialize_desktop_registration_random(random_source)
            .map_err(|_| StorageError::InvalidDesktopState("random initialization"))?;
        let compatibility = CompatibilityState {
            profile: CompatibilityProfile::Desktop644,
            random: RandomContinuation::Desktop644(random),
        };
        let import_metadata = DesktopImportMetadata::from_validated(&save.adaptations);
        let mut persisted_state = state.clone();
        persisted_state.profile = crate::desktop_save::DesktopValidatedProfile {
            motto: String::new(),
            guild: String::new(),
        };
        let canonical_state =
            serde_json::to_string(&persisted_state).map_err(StorageError::StateJson)?;
        let random_continuation =
            serde_json::to_string(&compatibility.random).map_err(StorageError::StateJson)?;
        let persisted_metadata =
            serde_json::to_string(&ImportMetadata::Desktop(import_metadata.clone()))
                .map_err(StorageError::StateJson)?;
        let id = CharacterId::new();
        let now = unix_millis();

        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        transaction.execute(
            "INSERT INTO characters (
                id, name, race, character_class, level, canonical_state,
                canonical_state_version, original_document, motto, guild,
                compatibility_profile, random_continuation, import_metadata,
                created_at_unix_ms, updated_at_unix_ms
             ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, 'null', ?8, ?9, ?10, ?11, ?12, ?13, ?14
             )",
            params![
                id.to_string(),
                identity.name,
                identity.race,
                identity.class,
                sqlite_integer(identity.level, "level")?,
                canonical_state,
                CANONICAL_STATE_VERSION,
                state.profile.motto,
                state.profile.guild,
                "desktop-6.4.4",
                random_continuation,
                persisted_metadata,
                now,
                now,
            ],
        )?;
        transaction.execute(
            "INSERT INTO desktop_private (
                character_id, passkey, realm, endpoint, account, password
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                id.to_string(),
                save.private.passkey,
                save.private.realm,
                save.private.endpoint,
                save.private.account,
                save.private.password,
            ],
        )?;

        #[cfg(test)]
        if std::mem::take(&mut self.fail_next_desktop_registration) {
            return Err(StorageError::InjectedFailure);
        }

        transaction.commit()?;
        Ok(RegisteredDesktopCharacter {
            id,
            identity,
            state_version: CANONICAL_STATE_VERSION,
            created_at_unix_ms: now,
            updated_at_unix_ms: now,
            state,
            compatibility,
            import_metadata,
        })
    }

    pub fn get_desktop(
        &self,
        id: &CharacterId,
    ) -> Result<RegisteredDesktopCharacter, StorageError> {
        self.connection
            .query_row(
                "SELECT id, name, race, character_class, level, canonical_state,
                        canonical_state_version, compatibility_profile,
                        random_continuation, import_metadata, motto, guild,
                        created_at_unix_ms, updated_at_unix_ms
                 FROM characters
                 WHERE id = ?1 AND compatibility_profile = 'desktop-6.4.4'",
                [id.to_string()],
                desktop_character_from_row,
            )
            .optional()?
            .ok_or_else(|| StorageError::NotFound(id.clone()))
    }

    pub fn compatibility_profile(
        &self,
        id: &CharacterId,
    ) -> Result<CompatibilityProfile, StorageError> {
        let (profile, random) = self
            .connection
            .query_row(
                "SELECT compatibility_profile, random_continuation
                 FROM characters WHERE id = ?1",
                [id.to_string()],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
            )
            .optional()?
            .ok_or_else(|| StorageError::NotFound(id.clone()))?;
        let profile = match profile.as_str() {
            "browser" => CompatibilityProfile::Browser,
            "desktop-6.4.4" => CompatibilityProfile::Desktop644,
            _ => {
                return Err(StorageError::InvalidCompatibilityState(
                    "unsupported compatibility profile",
                ));
            }
        };
        let random = serde_json::from_str(&random).map_err(StorageError::StateJson)?;
        CompatibilityState { profile, random }
            .validate()
            .map(|compatibility| compatibility.profile)
            .map_err(|_| {
                StorageError::InvalidCompatibilityState(
                    "profile and random continuation do not match",
                )
            })
    }

    pub fn identity(&self, id: &CharacterId) -> Result<CharacterIdentity, StorageError> {
        self.connection
            .query_row(
                "SELECT name, race, character_class, level
                 FROM characters WHERE id = ?1",
                [id.to_string()],
                |row| {
                    let level = row.get::<_, i64>(3)?;
                    Ok(CharacterIdentity {
                        name: row.get(0)?,
                        race: row.get(1)?,
                        class: row.get(2)?,
                        level: u64::try_from(level)
                            .map_err(|_| rusqlite::Error::IntegralValueOutOfRange(3, level))?,
                    })
                },
            )
            .optional()?
            .ok_or_else(|| StorageError::NotFound(id.clone()))
    }

    pub fn replace_desktop_state(
        &mut self,
        id: &CharacterId,
        state: &DesktopCanonicalState,
    ) -> Result<(), StorageError> {
        let identity = desktop_identity(state)?;
        let mut canonical = state.clone();
        canonical.profile = crate::desktop_save::DesktopValidatedProfile {
            motto: String::new(),
            guild: String::new(),
        };
        let canonical_state = serde_json::to_string(&canonical).map_err(StorageError::StateJson)?;
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let changed = transaction.execute(
            "UPDATE characters
             SET name = ?1, race = ?2, character_class = ?3, level = ?4,
                 canonical_state = ?5, canonical_state_version = ?6,
                 updated_at_unix_ms = ?7
             WHERE id = ?8 AND compatibility_profile = 'desktop-6.4.4'",
            params![
                identity.name,
                identity.race,
                identity.class,
                sqlite_integer(identity.level, "level")?,
                canonical_state,
                CANONICAL_STATE_VERSION,
                unix_millis(),
                id.to_string(),
            ],
        )?;
        if changed == 0 {
            return Err(StorageError::NotFound(id.clone()));
        }
        transaction.commit()?;
        Ok(())
    }

    fn replace_desktop_checkpoint(
        &mut self,
        id: &CharacterId,
        checkpoint: &DesktopCallbackCheckpoint,
        observation: DesktopCallbackObservation,
        mark_local_only: bool,
    ) -> Result<(), StorageError> {
        let identity = desktop_identity(&checkpoint.state)?;
        let mut canonical = checkpoint.state.clone();
        canonical.profile = crate::desktop_save::DesktopValidatedProfile {
            motto: String::new(),
            guild: String::new(),
        };
        let canonical_state = serde_json::to_string(&canonical).map_err(StorageError::StateJson)?;
        let random_continuation =
            serde_json::to_string(&RandomContinuation::Desktop644(checkpoint.random))
                .map_err(StorageError::StateJson)?;
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let persisted_metadata = transaction
            .query_row(
                "SELECT import_metadata FROM characters
                 WHERE id = ?1 AND compatibility_profile = 'desktop-6.4.4'",
                [id.to_string()],
                |row| row.get::<_, String>(0),
            )
            .optional()?
            .ok_or_else(|| StorageError::NotFound(id.clone()))?;
        let ImportMetadata::Desktop(mut metadata) =
            serde_json::from_str(&persisted_metadata).map_err(StorageError::StateJson)?
        else {
            return Err(StorageError::InvalidDesktopState(
                "import metadata profile mismatch",
            ));
        };
        metadata.measured_since_import.elapsed_milliseconds = metadata
            .measured_since_import
            .elapsed_milliseconds
            .checked_add(observation.credited_milliseconds)
            .ok_or(StorageError::DesktopCounterOverflow("elapsed milliseconds"))?;
        if observation.completion_dispatched {
            metadata.measured_since_import.tasks_completed = metadata
                .measured_since_import
                .tasks_completed
                .checked_add(1)
                .ok_or(StorageError::DesktopCounterOverflow("tasks completed"))?;
        }
        if mark_local_only {
            metadata.advancement_provenance = DesktopAdvancementProvenance::LocalOnly;
        }
        let import_metadata = serde_json::to_string(&ImportMetadata::Desktop(metadata))
            .map_err(StorageError::StateJson)?;

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
                 random_continuation = ?7, import_metadata = ?8,
                 updated_at_unix_ms = ?9
             WHERE id = ?10 AND compatibility_profile = 'desktop-6.4.4'",
            params![
                identity.name,
                identity.race,
                identity.class,
                sqlite_integer(identity.level, "level")?,
                canonical_state,
                CANONICAL_STATE_VERSION,
                random_continuation,
                import_metadata,
                unix_millis(),
                id.to_string(),
            ],
        )?;
        if changed == 0 {
            return Err(StorageError::NotFound(id.clone()));
        }
        transaction.commit()?;
        Ok(())
    }

    #[allow(dead_code)]
    pub(crate) fn desktop_authentication(
        &self,
        id: &CharacterId,
    ) -> Result<DesktopAuthentication, StorageError> {
        self.connection
            .query_row(
                "SELECT passkey, realm, endpoint, account, password
                 FROM desktop_private WHERE character_id = ?1",
                [id.to_string()],
                |row| {
                    Ok(DesktopAuthentication {
                        passkey: row.get(0)?,
                        realm: row.get(1)?,
                        endpoint: row.get(2)?,
                        account: row.get(3)?,
                        password: row.get(4)?,
                    })
                },
            )
            .optional()?
            .ok_or_else(|| StorageError::NotFound(id.clone()))
    }

    pub fn list(&self) -> Result<Vec<ManagedCharacter>, StorageError> {
        let mut statement = self.connection.prepare(
            "SELECT id, name, race, character_class, level, canonical_state,
                    canonical_state_version, motto, guild, created_at_unix_ms,
                    updated_at_unix_ms
             FROM characters
             WHERE compatibility_profile = 'browser'
             ORDER BY created_at_unix_ms, id",
        )?;
        let mut rows = statement.query([])?;
        let mut characters = Vec::new();
        while let Some(row) = rows.next()? {
            characters.push(managed_character_from_row(row)?);
        }
        Ok(characters)
    }

    pub fn list_managed(&self) -> Result<Vec<ManagedCharacterSummary>, StorageError> {
        let records = {
            let mut statement = self.connection.prepare(
                "SELECT id, name, race, character_class, level, updated_at_unix_ms
                 FROM characters
                 ORDER BY created_at_unix_ms, id",
            )?;
            let rows = statement.query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get(5)?,
                ))
            })?;
            rows.collect::<Result<Vec<_>, _>>()?
        };

        records
            .into_iter()
            .map(|(id, name, race, class, level, updated_at_unix_ms)| {
                let parsed_id =
                    CharacterId::parse(&id).map_err(|_| StorageError::InvalidCharacterId(id))?;
                let level =
                    u64::try_from(level).map_err(|_| StorageError::IntegerOutOfRange("level"))?;
                Ok(ManagedCharacterSummary {
                    compatibility: self.managed_compatibility(&parsed_id)?,
                    id: parsed_id,
                    identity: CharacterIdentity {
                        name,
                        race,
                        class,
                        level,
                    },
                    updated_at_unix_ms,
                })
            })
            .collect()
    }

    pub fn managed_compatibility(
        &self,
        id: &CharacterId,
    ) -> Result<ManagedCompatibilityPresentation, StorageError> {
        match self.compatibility_profile(id)? {
            CompatibilityProfile::Browser => Ok(ManagedCompatibilityPresentation {
                profile: CompatibilityProfile::Browser,
                realm: None,
                online_eligibility: Vec::new(),
                unavailable_history: None,
                measured_since_import: None,
                advancement_provenance: None,
                notice: None,
            }),
            CompatibilityProfile::Desktop644 => {
                let character = self.get_desktop(id)?;
                let authentication = self.desktop_authentication(id)?;
                let operations = [
                    DesktopOnlineOperation::AutomaticLevel,
                    DesktopOnlineOperation::AutomaticAct,
                    DesktopOnlineOperation::ManualBrag,
                    DesktopOnlineOperation::Motto,
                    DesktopOnlineOperation::Guild,
                ];
                let online_eligibility = operations
                    .into_iter()
                    .map(|operation| DesktopOperationEligibility {
                        operation,
                        decision: desktop_eligibility_decision(
                            &character,
                            &authentication,
                            &character.state,
                            operation,
                            None,
                        ),
                    })
                    .collect();
                let provenance = character.import_metadata.advancement_provenance;
                let notice = match provenance {
                    DesktopAdvancementProvenance::LocalOnly => Some(
                        "This desktop character is a local-only fork. Future online use requires a fresh official-client import."
                            .to_owned(),
                    ),
                    DesktopAdvancementProvenance::Unadvanced
                        if authentication.has_online_origin()
                            && !desktop_progress_reporting_evidence_available(
                                &character,
                                &authentication,
                            ) =>
                    {
                        Some(
                            "Starting local advancement while classic reporting is gated will permanently make this managed character local-only. Future online use will require a fresh official-client import."
                                .to_owned(),
                        )
                    }
                    DesktopAdvancementProvenance::Unadvanced => None,
                };
                Ok(ManagedCompatibilityPresentation {
                    profile: CompatibilityProfile::Desktop644,
                    realm: Some(authentication.realm),
                    online_eligibility,
                    unavailable_history: Some(
                        character.import_metadata.unavailable_history.clone(),
                    ),
                    measured_since_import: Some(
                        character.import_metadata.measured_since_import.clone(),
                    ),
                    advancement_provenance: Some(provenance),
                    notice,
                })
            }
        }
    }

    pub fn managed_inspection(&self, id: &CharacterId) -> Result<ManagedInspection, StorageError> {
        let compatibility = self.managed_compatibility(id)?;
        let state = match compatibility.profile {
            CompatibilityProfile::Browser => ManagedInspectionState::Browser(self.get(id)?),
            CompatibilityProfile::Desktop644 => {
                ManagedInspectionState::Desktop644(self.get_desktop(id)?)
            }
        };
        Ok(ManagedInspection {
            compatibility,
            state,
        })
    }

    pub fn start_warning(&self, id: &CharacterId) -> Result<Option<String>, StorageError> {
        Ok(self.managed_compatibility(id)?.notice)
    }

    pub fn get(&self, id: &CharacterId) -> Result<ManagedCharacter, StorageError> {
        let mut statement = self.connection.prepare(
            "SELECT id, name, race, character_class, level, canonical_state,
                    canonical_state_version, motto, guild, created_at_unix_ms,
                    updated_at_unix_ms
             FROM characters
             WHERE id = ?1 AND compatibility_profile = 'browser'",
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

    pub(crate) fn desktop_reporting_target_for_worker(
        &self,
        id: &CharacterId,
        operation: DesktopOnlineOperation,
        request_state: &DesktopCanonicalState,
    ) -> Result<DesktopReportingTarget, StorageError> {
        let online_action_lock = self.acquire_online_action_lock(id)?;
        let character = self.get_desktop(id)?;
        let authentication = self.desktop_authentication(id)?;
        ensure_desktop_online_eligible(
            &character,
            &authentication,
            request_state,
            operation,
            None,
        )?;
        let profile = OnlineProfile {
            motto: character.state.profile.motto.clone(),
            guild: character.state.profile.guild.clone(),
        };
        Ok(DesktopReportingTarget {
            identity: character.identity,
            state: character.state,
            adaptations: character.import_metadata.provenance.adaptations,
            profile,
            authentication,
            _online_action_lock: online_action_lock,
        })
    }

    pub(crate) fn desktop_online_action_target(
        &self,
        id: &CharacterId,
        operation: DesktopOnlineOperation,
        submitted_text: Option<&str>,
    ) -> Result<DesktopReportingTarget, StorageError> {
        let online_action_lock = self.acquire_online_action_lock(id)?;
        let character = self.get_desktop(id)?;
        let authentication = self.desktop_authentication(id)?;
        ensure_desktop_online_eligible(
            &character,
            &authentication,
            &character.state,
            operation,
            submitted_text,
        )?;
        let profile = OnlineProfile {
            motto: character.state.profile.motto.clone(),
            guild: character.state.profile.guild.clone(),
        };
        Ok(DesktopReportingTarget {
            identity: character.identity,
            state: character.state,
            adaptations: character.import_metadata.provenance.adaptations,
            profile,
            authentication,
            _online_action_lock: online_action_lock,
        })
    }

    /// Resolves an online credential while holding the short-lived online
    /// action lock, allowing explicit online actions during active simulation.
    pub(crate) fn online_action_target(
        &self,
        id: &CharacterId,
        operation: DesktopOnlineOperation,
        submitted_text: Option<&str>,
    ) -> Result<ReportingTarget, StorageError> {
        let online_action_lock = self.acquire_online_action_lock(id)?;
        match self.compatibility_profile(id)? {
            CompatibilityProfile::Browser => {}
            CompatibilityProfile::Desktop644 => {
                let character = self.get_desktop(id)?;
                let authentication = self.desktop_authentication(id)?;
                ensure_desktop_online_eligible(
                    &character,
                    &authentication,
                    &character.state,
                    operation,
                    submitted_text,
                )?;
                return Err(StorageError::ReportingIneligible);
            }
        }
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

        transaction.execute(
            "DELETE FROM desktop_private WHERE character_id = ?1",
            [id.to_string()],
        )?;

        #[cfg(test)]
        if std::mem::take(&mut self.fail_next_remove) {
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

    #[cfg(test)]
    fn inject_next_desktop_registration_failure(&mut self) {
        self.fail_next_desktop_registration = true;
    }
}

fn initialize_schema(connection: &mut Connection, data_root: &Path) -> Result<(), StorageError> {
    let observed_version: i64 =
        connection.pragma_query_value(None, "user_version", |row| row.get(0))?;
    if observed_version > DATABASE_SCHEMA_VERSION {
        return Err(StorageError::UnsupportedSchema(observed_version));
    }
    if observed_version > 0 && observed_version < DATABASE_SCHEMA_VERSION {
        create_migration_backup(connection, data_root, observed_version)?;
    }

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
                compatibility_profile TEXT NOT NULL,
                random_continuation TEXT NOT NULL,
                import_metadata TEXT NOT NULL,
                created_at_unix_ms INTEGER NOT NULL,
                updated_at_unix_ms INTEGER NOT NULL
            );",
        )?;
    } else {
        if version == 1 {
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
        if version <= 2 {
            transaction.execute_batch(
                "ALTER TABLE characters
                    ADD COLUMN compatibility_profile TEXT NOT NULL DEFAULT 'browser';
                 ALTER TABLE characters
                    ADD COLUMN random_continuation TEXT NOT NULL DEFAULT '';
                 ALTER TABLE characters
                    ADD COLUMN import_metadata TEXT NOT NULL DEFAULT '';",
            )?;
            let browser_rows = {
                let mut statement =
                    transaction.prepare("SELECT id, canonical_state FROM characters")?;
                let rows = statement.query_map([], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                })?;
                rows.collect::<Result<Vec<_>, _>>()?
            };
            for (id, canonical_state) in browser_rows {
                let character: Character =
                    serde_json::from_str(&canonical_state).map_err(StorageError::StateJson)?;
                let random = serde_json::to_string(&RandomContinuation::Browser(character.seed))
                    .map_err(StorageError::StateJson)?;
                let metadata = serde_json::to_string(&ImportMetadata::Browser {
                    source_format: SourceFormat::BrowserJson,
                })
                .map_err(StorageError::StateJson)?;
                transaction.execute(
                    "UPDATE characters
                     SET canonical_state_version = ?1, random_continuation = ?2,
                         import_metadata = ?3
                     WHERE id = ?4",
                    params![CANONICAL_STATE_VERSION, random, metadata, id],
                )?;
            }
        }
    }
    if version < 4 {
        transaction.execute_batch(
            "CREATE TABLE desktop_private (
                character_id TEXT PRIMARY KEY NOT NULL,
                passkey INTEGER NOT NULL,
                realm TEXT NOT NULL,
                endpoint TEXT NOT NULL,
                account TEXT NOT NULL,
                password TEXT NOT NULL,
                FOREIGN KEY(character_id) REFERENCES characters(id)
            );",
        )?;
    }
    transaction.pragma_update(None, "user_version", DATABASE_SCHEMA_VERSION)?;
    transaction.commit()?;
    Ok(())
}

fn create_migration_backup(
    connection: &Connection,
    data_root: &Path,
    source_version: i64,
) -> Result<(), StorageError> {
    let backup = data_root.join(format!("{DATABASE_FILENAME}.pre-v{source_version}.backup"));
    if !backup.exists() {
        connection.execute("VACUUM INTO ?1", [backup.to_string_lossy().as_ref()])?;
    }
    restrict_permissions(&backup, 0o600)
}

fn restrict_store_files(data_root: &Path) -> Result<(), StorageError> {
    for name in [
        DATABASE_FILENAME.to_owned(),
        format!("{DATABASE_FILENAME}-journal"),
        format!("{DATABASE_FILENAME}-wal"),
        format!("{DATABASE_FILENAME}-shm"),
    ] {
        let path = data_root.join(name);
        if path.exists() {
            restrict_permissions(&path, 0o600)?;
        }
    }
    for entry in fs::read_dir(data_root)? {
        let path = entry?.path();
        if path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| {
                name.starts_with(&format!("{DATABASE_FILENAME}.pre-v")) && name.ends_with(".backup")
            })
        {
            restrict_permissions(&path, 0o600)?;
        }
    }
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

fn desktop_character_from_row(
    row: &rusqlite::Row<'_>,
) -> Result<RegisteredDesktopCharacter, rusqlite::Error> {
    desktop_character_from_row_inner(row).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(5, rusqlite::types::Type::Text, Box::new(error))
    })
}

fn desktop_character_from_row_inner(
    row: &rusqlite::Row<'_>,
) -> Result<RegisteredDesktopCharacter, StorageError> {
    let profile: String = row.get(7)?;
    if profile != "desktop-6.4.4" {
        return Err(StorageError::InvalidDesktopState(
            "compatibility profile is not desktop-6.4.4",
        ));
    }
    let mut state: DesktopCanonicalState =
        serde_json::from_str(&row.get::<_, String>(5)?).map_err(StorageError::StateJson)?;
    state.profile = crate::desktop_save::DesktopValidatedProfile {
        motto: row.get(10)?,
        guild: row.get(11)?,
    };
    let random: RandomContinuation =
        serde_json::from_str(&row.get::<_, String>(8)?).map_err(StorageError::StateJson)?;
    let compatibility = CompatibilityState {
        profile: CompatibilityProfile::Desktop644,
        random,
    }
    .validate()
    .map_err(|_| StorageError::InvalidDesktopState("random continuation profile mismatch"))?;
    let metadata: ImportMetadata =
        serde_json::from_str(&row.get::<_, String>(9)?).map_err(StorageError::StateJson)?;
    let ImportMetadata::Desktop(import_metadata) = metadata else {
        return Err(StorageError::InvalidDesktopState(
            "import metadata profile mismatch",
        ));
    };
    Ok(RegisteredDesktopCharacter {
        id: CharacterId::parse(&row.get::<_, String>(0)?)?,
        identity: CharacterIdentity {
            name: row.get(1)?,
            race: row.get(2)?,
            class: row.get(3)?,
            level: u64::try_from(row.get::<_, i64>(4)?)
                .map_err(|_| StorageError::IntegerOutOfRange("level"))?,
        },
        state_version: row.get(6)?,
        created_at_unix_ms: row.get(12)?,
        updated_at_unix_ms: row.get(13)?,
        state,
        compatibility,
        import_metadata,
    })
}

fn canonical_json(character: &Character) -> Result<String, StorageError> {
    let mut canonical = character.clone();
    canonical.profile = OnlineProfile::default();
    serde_json::to_string(&canonical).map_err(StorageError::StateJson)
}

fn desktop_identity(state: &DesktopCanonicalState) -> Result<CharacterIdentity, StorageError> {
    let value = |index: usize, caption: &'static str| {
        state
            .traits
            .get(index)
            .filter(|row| row.caption == caption && row.subitems.len() == 1)
            .and_then(|row| row.subitems.first())
            .ok_or(StorageError::InvalidDesktopState("trait layout"))
    };
    Ok(CharacterIdentity {
        name: value(0, "Name")?.clone(),
        race: value(1, "Race")?.clone(),
        class: value(2, "Class")?.clone(),
        level: value(3, "Level")?
            .parse()
            .map_err(|_| StorageError::InvalidDesktopState("level"))?,
    })
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
    #[error(transparent)]
    DesktopCallback(#[from] crate::desktop_callback::DesktopCallbackError),
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
    profile: CompatibilityProfile,
    _lock: CharacterLock,
    last_tick: Instant,
    transport: Box<dyn ReportTransport>,
}

impl Worker {
    /// Takes ownership before reading any persisted character state.
    pub fn start(store: Store, id: CharacterId) -> Result<Self, WorkerError> {
        let lock = store.acquire_lock(&id)?;
        let profile = store.compatibility_profile(&id)?;
        Ok(Self {
            store,
            id,
            profile,
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
        match self.profile {
            CompatibilityProfile::Browser => {
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
                    let _ = crate::reporting::submit_event(
                        &self.store,
                        &self.id,
                        event,
                        &*self.transport,
                    );
                }
            }
            CompatibilityProfile::Desktop644 => {
                let managed = self.store.get_desktop(&self.id)?;
                let RandomContinuation::Desktop644(random) = managed.compatibility.random else {
                    return Err(StorageError::InvalidDesktopState(
                        "random continuation profile mismatch",
                    )
                    .into());
                };
                let authentication = self.store.desktop_authentication(&self.id)?;
                let progress_reporting_evidence_available =
                    desktop_progress_reporting_evidence_available(&managed, &authentication);
                let mut checkpoint = DesktopCallbackCheckpoint {
                    state: managed.state,
                    random,
                };
                let mut hooks = SourceDerivedDesktopHooks::traced();
                let elapsed_ms =
                    i64::try_from(elapsed_ms).map_err(|_| WorkerError::ElapsedOverflow)?;
                let observation = checkpoint.apply_progression_callback(elapsed_ms, &mut hooks)?;
                let changed =
                    observation.credited_milliseconds != 0 || observation.completion_dispatched;
                let mark_local_only = should_mark_desktop_local_only(
                    &authentication,
                    changed,
                    progress_reporting_evidence_available,
                );
                self.store.replace_desktop_checkpoint(
                    &self.id,
                    &checkpoint,
                    observation,
                    mark_local_only,
                )?;
                for report in hooks.reports() {
                    let _ = crate::reporting::submit_desktop_event(
                        &self.store,
                        &self.id,
                        report,
                        &*self.transport,
                    );
                }
            }
        }
        Ok(())
    }

    fn scheduled_duration(&self, interval: Duration) -> Result<Duration, WorkerError> {
        match self.profile {
            CompatibilityProfile::Browser => {
                let state = self.store.get(&self.id)?.state;
                Ok(aligned_task_completion_duration(&state).min(interval))
            }
            CompatibilityProfile::Desktop644 => Ok(Duration::from_millis(
                crate::desktop_callback::MAX_CALLBACK_ELAPSED_MS as u64,
            )),
        }
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
            let scheduled = self.scheduled_duration(interval)?;
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
    use std::sync::{Arc, Barrier, Mutex, mpsc};
    use url::Url;

    use super::*;
    use crate::{
        checkpoint,
        desktop_save::{
            DesktopAdaptations, DesktopQuestMarker, DesktopQueueCommand, DesktopQueueKind,
            DesktopValidatedBar, DesktopValidatedBars, DesktopValidatedPrivateMetadata,
            DesktopValidatedProfile, DesktopValidatedRow,
        },
        desktop_simulation::{DesktopReportSnapshot, DesktopReportTrigger},
        guild::{GuildOutcome, GuildResponseRules},
        reporting::{
            DeliveryOutcome, GuildTransport, OFFICIAL_LEADERBOARD_ENDPOINT, ReportTransport,
            ReportingError,
        },
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

    fn desktop_fixture() -> DesktopValidatedSave {
        let rows = |values: &[(&str, &str)]| {
            values
                .iter()
                .map(|(caption, value)| DesktopValidatedRow {
                    header: [0, -1, -1, 0, 1],
                    caption: (*caption).to_owned(),
                    subitems: vec![(*value).to_owned()],
                })
                .collect()
        };
        DesktopValidatedSave {
            traits: rows(&[
                ("Name", "Desktop Hero"),
                ("Race", "Gyrognome"),
                ("Class", "Robot Monk"),
                ("Level", "2"),
            ]),
            stats: rows(&[
                ("STR", "12"),
                ("CON", "11"),
                ("DEX", "10"),
                ("INT", "9"),
                ("WIS", "8"),
                ("CHA", "7"),
                ("HP Max", "20"),
                ("MP Max", "15"),
            ]),
            equipment: rows(&[
                ("Weapon", "Stick"),
                ("Shield", "Plate"),
                ("Helm", "Cap"),
                ("Hauberk", "Burlap"),
                ("Brassairts", "Cloth"),
                ("Vambraces", "Cloth"),
                ("Gauntlets", "Cloth"),
                ("Gambeson", "Cloth"),
                ("Cuisses", "Cloth"),
                ("Greaves", "Cloth"),
                ("Sollerets", "Cloth"),
            ]),
            inventory: rows(&[("Gold", "3")]),
            spells: Vec::new(),
            plots: Vec::new(),
            quests: Vec::new(),
            current_task: "load".to_owned(),
            quest: DesktopQuestMarker::None,
            queue: Vec::new(),
            activity: "Loading...".to_owned(),
            bars: DesktopValidatedBars {
                experience: DesktopValidatedBar {
                    position: 0,
                    maximum: 100,
                },
                encumbrance: DesktopValidatedBar {
                    position: 0,
                    maximum: 50,
                },
                plot: DesktopValidatedBar {
                    position: 0,
                    maximum: 26,
                },
                quest: DesktopValidatedBar {
                    position: 0,
                    maximum: 0,
                },
                task: DesktopValidatedBar {
                    position: 0,
                    maximum: 2_000,
                },
            },
            prized_equipment: 0,
            game_style: 3,
            profile: DesktopValidatedProfile {
                motto: "Synthetic motto".to_owned(),
                guild: "Synthetic Guild".to_owned(),
            },
            private: DesktopValidatedPrivateMetadata {
                passkey: 4_242,
                realm: "Synthetic Realm".to_owned(),
                endpoint: "https://synthetic.invalid/".to_owned(),
                account: "synthetic-account".to_owned(),
                password: "synthetic-password".to_owned(),
            },
            adaptations: DesktopAdaptations {
                legacy_prologue_62: false,
                legacy_quest_placeholder: false,
                spelling_patch_applied: false,
            },
        }
    }

    fn desktop_level_report_fixture() -> DesktopValidatedSave {
        let mut save = desktop_fixture();
        save.current_task = "kill|Rat|1|tail".to_owned();
        save.activity = "Executing Rat...".to_owned();
        save.bars.task = DesktopValidatedBar {
            position: 6_000,
            maximum: 6_000,
        };
        save.bars.experience.position = save.bars.experience.maximum;
        save.queue = vec![DesktopQueueCommand {
            kind: DesktopQueueKind::Task,
            duration_seconds: 2,
            caption: "Continue".to_owned(),
        }];
        save
    }

    fn evidenced_spoltog_desktop_fixture() -> DesktopValidatedSave {
        let mut save = desktop_fixture();
        save.private.realm = "Spoltog".to_owned();
        save.private.endpoint = "http://progressquest.com/spoltog.php?".to_owned();
        save.plots = vec![DesktopValidatedRow {
            header: [0, -1, -1, 0, 1],
            caption: "Act I".to_owned(),
            subitems: vec!["0".to_owned()],
        }];
        save
    }

    fn pemptus_fixture() -> DesktopValidatedSave {
        let mut save = evidenced_spoltog_desktop_fixture();
        save.private.realm = crate::desktop_contract::PEMPTUS.realm.to_owned();
        save.private.endpoint = crate::desktop_contract::PEMPTUS.saved_endpoint.to_owned();
        save.private.account.clear();
        save.private.password.clear();
        save
    }

    fn pemptus_placeholder_fixture() -> DesktopValidatedSave {
        let mut save = pemptus_fixture();
        save.adaptations.legacy_quest_placeholder = true;
        save.quest = DesktopQuestMarker::LegacyPlaceholder { index: 1 };
        save
    }

    struct PemptusTransport {
        path: PathBuf,
        id: CharacterId,
        calls: Arc<Mutex<Vec<String>>>,
        fail: bool,
    }

    impl ReportTransport for PemptusTransport {
        fn deliver(&self, _: Url) -> DeliveryOutcome {
            panic!("Pemptus used browser transport")
        }

        fn deliver_desktop(
            &self,
            target: &crate::desktop_transport::VerifiedDesktopEndpoint,
            query: &str,
            credentials: &crate::desktop_transport::DesktopTransportCredentials,
        ) -> Result<
            crate::desktop_transport::DesktopHttpResponse,
            crate::desktop_transport::DesktopTransportError,
        > {
            assert_eq!(
                target.credential_mode(),
                crate::desktop_eligibility::DesktopCredentialMode::PasskeyOnly
            );
            assert!(
                credentials
                    .authorization_header(target.credential_mode())
                    .unwrap()
                    .is_none()
            );
            let persisted = Store::open_at(&self.path)
                .unwrap()
                .get_desktop(&self.id)
                .unwrap();
            let fields: Vec<_> = url::form_urlencoded::parse(query.as_bytes()).collect();
            let field = |name| {
                fields
                    .iter()
                    .find(|(key, _)| key == name)
                    .unwrap()
                    .1
                    .to_string()
            };
            assert_eq!(field("h"), "Pemptus");
            assert_eq!(field("l"), persisted.identity.level.to_string());
            assert_eq!(field("m"), persisted.state.profile.motto);
            self.calls.lock().unwrap().push(field("t"));
            if self.fail {
                Err(crate::desktop_transport::DesktopTransportError::DeliveryFailed)
            } else {
                Ok(crate::desktop_transport::DesktopHttpResponse {
                    status: 200,
                    redirect: None,
                    body: vec![],
                })
            }
        }
    }

    impl GuildTransport for PemptusTransport {
        fn guild(&self, _: Url, _: &str, _: &str, _: &GuildResponseRules) -> GuildOutcome {
            panic!("Pemptus used browser guild transport")
        }

        fn guild_desktop(
            &self,
            target: &crate::desktop_transport::VerifiedDesktopEndpoint,
            query: &str,
            credentials: &crate::desktop_transport::DesktopTransportCredentials,
        ) -> Result<
            crate::desktop_transport::DesktopHttpResponse,
            crate::desktop_transport::DesktopTransportError,
        > {
            assert!(
                credentials
                    .authorization_header(target.credential_mode())
                    .unwrap()
                    .is_none()
            );
            let fields: Vec<_> = url::form_urlencoded::parse(query.as_bytes()).collect();
            assert_eq!(
                fields.iter().find(|(key, _)| key == "h").unwrap().1,
                "Pemptus"
            );
            let guild = fields
                .iter()
                .find(|(key, _)| key == "guild")
                .unwrap()
                .1
                .as_ref();
            self.calls.lock().unwrap().push(format!("guild:{guild}"));
            if self.fail {
                return Err(crate::desktop_transport::DesktopTransportError::DeliveryFailed);
            }
            Ok(crate::desktop_transport::DesktopHttpResponse {
                status: 200,
                redirect: None,
                body: match guild {
                    "Guild A" => b"joined".to_vec(),
                    "Guild B" => b"changed".to_vec(),
                    "" => b"left".to_vec(),
                    _ => b"rejected".to_vec(),
                },
            })
        }

        fn desktop_public_guild(
            &self,
            target: &crate::desktop_transport::VerifiedDesktopEndpoint,
            name: &str,
        ) -> Result<Option<String>, crate::desktop_transport::DesktopTransportError> {
            assert_eq!(target.realm(), "Pemptus");
            let stored = Store::open_at(&self.path)
                .unwrap()
                .get_desktop(&self.id)
                .unwrap();
            assert_eq!(name, stored.identity.name);
            let mut calls = self.calls.lock().unwrap();
            let submitted = calls
                .iter()
                .rev()
                .find_map(|call| call.strip_prefix("guild:"))
                .unwrap()
                .to_owned();
            calls.push(format!("public:{name}"));
            Ok(match submitted.as_str() {
                "Guild A" | "Guild B" => Some(submitted),
                "" => None,
                _ if stored.state.profile.guild.is_empty() => None,
                _ => Some(stored.state.profile.guild),
            })
        }
    }

    struct Numbers(u32);

    impl RandomSource for Numbers {
        fn next_u32(&mut self) -> Result<u32, crate::newguy::NewGuyError> {
            Ok(self.0)
        }
    }

    struct RejectNetworkTransport;

    impl ReportTransport for RejectNetworkTransport {
        fn deliver(&self, _request: Url) -> DeliveryOutcome {
            panic!("gated desktop operation attempted network delivery")
        }
    }

    impl GuildTransport for RejectNetworkTransport {
        fn guild(
            &self,
            _request: Url,
            _prior: &str,
            _submitted: &str,
            _rules: &GuildResponseRules,
        ) -> GuildOutcome {
            panic!("gated desktop guild operation attempted network delivery")
        }
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

    fn create_version_two_database(
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
                    motto TEXT NOT NULL DEFAULT '',
                    guild TEXT NOT NULL DEFAULT '',
                    created_at_unix_ms INTEGER NOT NULL,
                    updated_at_unix_ms INTEGER NOT NULL
                );
                PRAGMA user_version = 2;",
            )
            .unwrap();
        let id = CharacterId::new();
        let canonical_state = canonical_json(character).unwrap();
        let original_document = serde_json::to_string(&character.document).unwrap();
        connection
            .execute(
                "INSERT INTO characters (
                    id, name, race, character_class, level, canonical_state,
                    canonical_state_version, original_document, motto, guild,
                    created_at_unix_ms, updated_at_unix_ms
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1, ?7, ?8, ?9, 1, 2)",
                params![
                    id.to_string(),
                    character.traits.name,
                    character.traits.race,
                    character.traits.class,
                    sqlite_integer(character.traits.level, "level").unwrap(),
                    canonical_state,
                    original_document,
                    character.profile.motto,
                    character.profile.guild,
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
    fn migrates_schema_two_browser_rows_without_changing_state_or_credentials() {
        let directory = TestDirectory::new("typed-profile-migration");
        let character = fixture_character();
        let expected_seed = character.seed;
        let expected_profile = character.profile.clone();
        let (id, canonical_state, original_document) =
            create_version_two_database(&directory.0, &character);

        let store = Store::open_at(&directory.0).unwrap();
        let persisted: (String, String, String, String, i64, String, String) = store
            .connection
            .query_row(
                "SELECT canonical_state, original_document, motto, guild,
                        canonical_state_version, random_continuation, import_metadata
                 FROM characters WHERE id = ?1",
                [id.to_string()],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                        row.get(5)?,
                        row.get(6)?,
                    ))
                },
            )
            .unwrap();

        assert_eq!(persisted.0, canonical_state);
        assert_eq!(persisted.1, original_document);
        assert_eq!(persisted.2, expected_profile.motto);
        assert_eq!(persisted.3, expected_profile.guild);
        assert_eq!(persisted.4, i64::from(CANONICAL_STATE_VERSION));
        assert_eq!(
            serde_json::from_str::<RandomContinuation>(&persisted.5).unwrap(),
            RandomContinuation::Browser(expected_seed)
        );
        assert_eq!(
            serde_json::from_str::<ImportMetadata>(&persisted.6).unwrap(),
            ImportMetadata::Browser {
                source_format: SourceFormat::BrowserJson
            }
        );
        assert_eq!(
            serde_json::from_str::<Value>(&persisted.1).unwrap()["online"]["passkey"],
            4242
        );
        assert_eq!(store.get(&id).unwrap().state.seed, expected_seed);

        let backup_path = directory
            .0
            .join(format!("{DATABASE_FILENAME}.pre-v2.backup"));
        assert_eq!(
            fs::metadata(&backup_path).unwrap().permissions().mode() & 0o777,
            0o600
        );
        let backup = Connection::open(backup_path).unwrap();
        let backup_version: i64 = backup
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .unwrap();
        let backup_state: (String, String) = backup
            .query_row(
                "SELECT canonical_state, original_document FROM characters WHERE id = ?1",
                [id.to_string()],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(backup_version, 2);
        assert_eq!(backup_state, (canonical_state, original_document));
    }

    #[test]
    fn registers_desktop_state_atomically_without_persisting_private_metadata() {
        let directory = TestDirectory::new("desktop-registration");
        let mut store = Store::open_at(&directory.0).unwrap();
        let save = desktop_fixture();
        let original_save = save.clone();
        let inspection = crate::save::inspect_desktop(&save);
        assert_eq!(save, original_save);
        assert_eq!(inspection.target_profile, CompatibilityProfile::Desktop644);
        assert_eq!(
            inspection.import_metadata.unavailable_history,
            crate::compatibility::DesktopUnavailableHistory {
                original_random_continuation: crate::compatibility::HistoricalValue::Unavailable,
                birthday: crate::compatibility::HistoricalValue::Unavailable,
                seed_history: crate::compatibility::HistoricalValue::Unavailable,
                lifetime_tasks: crate::compatibility::HistoricalValue::Unavailable,
                lifetime_elapsed: crate::compatibility::HistoricalValue::Unavailable,
            }
        );
        assert_eq!(
            inspection.import_metadata.measured_since_import,
            crate::compatibility::SinceImportCounters {
                tasks_completed: 0,
                elapsed_milliseconds: 0,
            }
        );
        assert_eq!(inspection.online_eligibility.len(), 5);
        assert!(inspection.online_eligibility.iter().all(|eligibility| {
            eligibility.decision
                == DesktopEligibilityDecision::Ineligible(DesktopIneligibilityReason::RealmMismatch)
        }));
        let safe_inspection = serde_json::to_string(&inspection).unwrap();
        for private in [
            "4242",
            "Synthetic Realm",
            "synthetic.invalid",
            "synthetic-account",
            "synthetic-password",
        ] {
            assert!(!safe_inspection.contains(private));
        }
        let registered = store
            .register_desktop(&save, &mut Numbers(0xf00d_cafe))
            .unwrap();
        assert_eq!(save, original_save);

        assert_eq!(registered.identity.name, "Desktop Hero");
        assert_eq!(
            registered.compatibility.random,
            RandomContinuation::Desktop644(crate::compatibility::DesktopRandomState(0xf00d_cafe))
        );
        assert!(store.list().unwrap().is_empty());
        let restored = store.get_desktop(&registered.id).unwrap();
        assert_eq!(restored.state, registered.state);
        assert_eq!(restored.compatibility, registered.compatibility);
        assert_eq!(restored.import_metadata, registered.import_metadata);
        assert_eq!(
            restored.import_metadata.advancement_provenance,
            DesktopAdvancementProvenance::Unadvanced
        );
        let authentication = store.desktop_authentication(&registered.id).unwrap();
        assert_eq!(authentication.passkey, 4_242);
        assert_eq!(authentication.realm, "Synthetic Realm");
        assert_eq!(authentication.endpoint, "https://synthetic.invalid/");
        assert_eq!(authentication.account, "synthetic-account");
        assert_eq!(authentication.password, "synthetic-password");

        let persisted: (String, String) = store
            .connection
            .query_row(
                "SELECT canonical_state, original_document
                 FROM characters WHERE id = ?1",
                [registered.id.to_string()],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(persisted.1, "null");
        for private in [
            "4242",
            "synthetic-account",
            "synthetic-password",
            "synthetic.invalid",
        ] {
            assert!(!persisted.0.contains(private));
        }
        let safe = serde_json::to_string(&registered).unwrap();
        let debug = format!("{authentication:?}");
        for private in [
            "4242",
            "Synthetic Realm",
            "synthetic.invalid",
            "synthetic-account",
            "synthetic-password",
        ] {
            assert!(!safe.contains(private));
            assert!(!debug.contains(private));
        }

        store.inject_next_desktop_registration_failure();
        assert!(matches!(
            store.register_desktop(&save, &mut Numbers(7)),
            Err(StorageError::InjectedFailure)
        ));
        let count: i64 = store
            .connection
            .query_row("SELECT COUNT(*) FROM characters", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn managed_presentations_include_desktop_profile_gates_without_private_metadata() {
        let directory = TestDirectory::new("desktop-managed-presentation");
        let mut store = Store::open_at(&directory.0).unwrap();
        let browser = store.register(&fixture_character()).unwrap();
        let desktop = store
            .register_desktop(&desktop_fixture(), &mut Numbers(42))
            .unwrap();

        let listed = store.list_managed().unwrap();
        assert_eq!(listed.len(), 2);
        assert_eq!(
            listed
                .iter()
                .find(|entry| entry.id == browser.id)
                .unwrap()
                .compatibility
                .profile,
            CompatibilityProfile::Browser
        );
        let desktop_summary = listed.iter().find(|entry| entry.id == desktop.id).unwrap();
        assert_eq!(
            desktop_summary.compatibility.profile,
            CompatibilityProfile::Desktop644
        );
        assert_eq!(desktop_summary.compatibility.online_eligibility.len(), 5);
        assert!(
            desktop_summary
                .compatibility
                .notice
                .as_deref()
                .is_some_and(|notice| notice.contains("permanently")
                    && notice.contains("fresh official-client"))
        );

        let inspection =
            serde_json::to_string(&store.managed_inspection(&desktop.id).unwrap()).unwrap();
        assert!(inspection.contains("desktop-6.4.4"));
        assert!(inspection.contains("realm-mismatch"));
        assert!(inspection.contains("unavailableHistory"));
        assert!(inspection.contains("measuredSinceImport"));
        for private in [
            "4242",
            "synthetic-account",
            "synthetic-password",
            "synthetic.invalid",
        ] {
            assert!(!inspection.contains(private));
        }
    }

    #[test]
    fn local_only_managed_presentation_requires_a_fresh_official_client_import() {
        let directory = TestDirectory::new("desktop-local-only-presentation");
        let mut store = Store::open_at(&directory.0).unwrap();
        let desktop = store
            .register_desktop(&desktop_fixture(), &mut Numbers(42))
            .unwrap();
        let mut metadata = desktop.import_metadata;
        metadata.advancement_provenance = DesktopAdvancementProvenance::LocalOnly;
        store
            .connection
            .execute(
                "UPDATE characters SET import_metadata = ?1 WHERE id = ?2",
                params![
                    serde_json::to_string(&ImportMetadata::Desktop(metadata)).unwrap(),
                    desktop.id.to_string()
                ],
            )
            .unwrap();

        let presentation = store.managed_compatibility(&desktop.id).unwrap();
        assert!(presentation.notice.as_deref().is_some_and(
            |notice| notice.contains("local-only") && notice.contains("fresh official-client")
        ));
        assert!(presentation.online_eligibility.iter().all(|eligibility| {
            eligibility.decision
                == DesktopEligibilityDecision::Ineligible(
                    DesktopIneligibilityReason::FreshOfficialClientImportRequired,
                )
        }));
    }

    #[test]
    fn desktop_profile_and_state_updates_preserve_private_authentication() {
        let directory = TestDirectory::new("desktop-profile-state-independence");
        let mut registration_store = Store::open_at(&directory.0).unwrap();
        let registered = registration_store
            .register_desktop(&desktop_fixture(), &mut Numbers(42))
            .unwrap();
        let expected_authentication = registration_store
            .desktop_authentication(&registered.id)
            .unwrap();

        let profile = OnlineProfile {
            motto: "Persist independently".to_owned(),
            guild: "Desktop Gnomes".to_owned(),
        };
        let mut next = registered.state.clone();
        next.activity = "Continuing locally".to_owned();

        let barrier = Arc::new(Barrier::new(3));
        let profile_path = directory.0.clone();
        let profile_id = registered.id.clone();
        let profile_barrier = Arc::clone(&barrier);
        let profile_update = profile.clone();
        let profile_thread = thread::spawn(move || {
            let mut store = Store::open_at(profile_path).unwrap();
            profile_barrier.wait();
            store.replace_profile(&profile_id, &profile_update).unwrap();
        });
        let state_path = directory.0.clone();
        let state_id = registered.id.clone();
        let state_barrier = Arc::clone(&barrier);
        let state_thread = thread::spawn(move || {
            let mut store = Store::open_at(state_path).unwrap();
            state_barrier.wait();
            store.replace_desktop_state(&state_id, &next).unwrap();
        });
        barrier.wait();
        profile_thread.join().unwrap();
        state_thread.join().unwrap();

        let reopened = Store::open_at(&directory.0).unwrap();
        let restored = reopened.get_desktop(&registered.id).unwrap();
        assert_eq!(restored.state.activity, "Continuing locally");
        assert_eq!(restored.state.profile.motto, profile.motto);
        assert_eq!(restored.state.profile.guild, profile.guild);
        assert_eq!(
            reopened.desktop_authentication(&registered.id).unwrap(),
            expected_authentication
        );
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
        let target = store
            .online_action_target(&registered.id, DesktopOnlineOperation::ManualBrag, None)
            .unwrap();
        assert_eq!(target.identity, registered.identity);
        assert_eq!(target.state.document, Value::Null);
        drop(target);

        let mut offline = fixture_character();
        offline.online = None;
        let offline = store.register(&offline).unwrap();
        assert!(matches!(
            store.online_action_target(&offline.id, DesktopOnlineOperation::ManualBrag, None),
            Err(StorageError::ReportingIneligible)
        ));

        assert!(matches!(
            store.online_action_target(
                &CharacterId::new(),
                DesktopOnlineOperation::ManualBrag,
                None
            ),
            Err(StorageError::NotFound(_))
        ));

        let owned = store.register(&fixture_character()).unwrap();
        let worker =
            Worker::start(Store::open_at(&directory.0).unwrap(), owned.id.clone()).unwrap();
        let target = store
            .online_action_target(&owned.id, DesktopOnlineOperation::ManualBrag, None)
            .unwrap();
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
            store.online_action_target(&registered.id, DesktopOnlineOperation::ManualBrag, None),
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
            store.online_action_target(&registered.id, DesktopOnlineOperation::ManualBrag, None),
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
    fn removes_desktop_canonical_and_private_rows_atomically() {
        let directory = TestDirectory::new("desktop-remove");
        let mut store = Store::open_at(&directory.0).unwrap();
        let registered = store
            .register_desktop(&desktop_fixture(), &mut Numbers(42))
            .unwrap();
        assert!(store.desktop_authentication(&registered.id).is_ok());

        store.remove(&registered.id).unwrap();

        assert!(matches!(
            store.get_desktop(&registered.id),
            Err(StorageError::NotFound(id)) if id == registered.id
        ));
        assert!(matches!(
            store.desktop_authentication(&registered.id),
            Err(StorageError::NotFound(id)) if id == registered.id
        ));
    }

    #[test]
    fn failed_desktop_remove_keeps_canonical_and_private_rows() {
        let directory = TestDirectory::new("desktop-remove-transaction");
        let mut store = Store::open_at(&directory.0).unwrap();
        let registered = store
            .register_desktop(&desktop_fixture(), &mut Numbers(42))
            .unwrap();
        store.inject_next_remove_failure();

        assert!(matches!(
            store.remove(&registered.id),
            Err(StorageError::InjectedFailure)
        ));
        drop(store);

        let reopened = Store::open_at(&directory.0).unwrap();
        assert!(reopened.get_desktop(&registered.id).is_ok());
        assert!(reopened.desktop_authentication(&registered.id).is_ok());
    }

    #[test]
    fn reopening_restricts_sqlite_sidecar_permissions() {
        let directory = TestDirectory::new("sidecar-permissions");
        drop(Store::open_at(&directory.0).unwrap());
        for suffix in ["-journal", "-wal", "-shm"] {
            let path = directory.0.join(format!("{DATABASE_FILENAME}{suffix}"));
            fs::write(&path, b"synthetic sidecar").unwrap();
            fs::set_permissions(&path, fs::Permissions::from_mode(0o666)).unwrap();
        }

        drop(Store::open_at(&directory.0).unwrap());

        for suffix in ["-journal", "-wal", "-shm"] {
            let path = directory.0.join(format!("{DATABASE_FILENAME}{suffix}"));
            if path.exists() {
                assert_eq!(
                    fs::metadata(path).unwrap().permissions().mode() & 0o777,
                    0o600
                );
            }
        }
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
    fn desktop_worker_uses_one_capped_callback_per_explicit_advance() {
        let directory = TestDirectory::new("desktop-worker-callback");
        let mut store = Store::open_at(&directory.0).unwrap();
        let mut save = desktop_fixture();
        save.bars.task.maximum = 6_000;
        let registered = store
            .register_desktop(&save, &mut Numbers(0xf00d_cafe))
            .unwrap();
        let mut worker =
            Worker::start(Store::open_at(&directory.0).unwrap(), registered.id.clone()).unwrap();

        assert_eq!(
            worker.scheduled_duration(Duration::from_secs(1)).unwrap(),
            Duration::from_millis(100)
        );
        worker.advance_elapsed(Duration::from_millis(250)).unwrap();
        assert_eq!(
            worker
                .store
                .get_desktop(&registered.id)
                .unwrap()
                .state
                .bars
                .task
                .position,
            100
        );
        worker.advance_elapsed(Duration::from_millis(100)).unwrap();
        let persisted = worker.store.get_desktop(&registered.id).unwrap();
        assert_eq!(persisted.state.bars.task.position, 200);
        assert_eq!(
            persisted.import_metadata.measured_since_import,
            crate::compatibility::SinceImportCounters {
                tasks_completed: 0,
                elapsed_milliseconds: 200,
            }
        );
        assert_eq!(
            persisted.import_metadata.advancement_provenance,
            DesktopAdvancementProvenance::LocalOnly
        );
    }

    #[test]
    fn desktop_worker_persists_full_bar_until_the_next_actual_callback() {
        let directory = TestDirectory::new("desktop-worker-pending-completion");
        let mut store = Store::open_at(&directory.0).unwrap();
        let mut save = desktop_fixture();
        save.bars.task.maximum = 6_000;
        let registered = store
            .register_desktop(&save, &mut Numbers(0xf00d_cafe))
            .unwrap();
        let initial_task = registered.state.current_task;
        let mut worker =
            Worker::start(Store::open_at(&directory.0).unwrap(), registered.id.clone()).unwrap();

        for _ in 0..60 {
            worker.advance_elapsed(Duration::from_secs(5)).unwrap();
        }
        let pending = worker.store.get_desktop(&registered.id).unwrap();
        assert_eq!(pending.state.bars.task.position, 6_000);
        assert_eq!(pending.state.current_task, initial_task);

        worker.advance_elapsed(Duration::from_millis(100)).unwrap();
        let completed = worker.store.get_desktop(&registered.id).unwrap();
        assert_eq!(completed.state.bars.task.position, 0);
        assert_ne!(completed.state.current_task, initial_task);
    }

    #[test]
    fn desktop_worker_restart_preserves_pending_completion_without_downtime() {
        let directory = TestDirectory::new("desktop-worker-restart");
        let mut store = Store::open_at(&directory.0).unwrap();
        let mut save = desktop_fixture();
        save.bars.task.position = 5_900;
        save.bars.task.maximum = 6_000;
        let registered = store
            .register_desktop(&save, &mut Numbers(0xf00d_cafe))
            .unwrap();
        let initial_task = registered.state.current_task;
        let mut worker =
            Worker::start(Store::open_at(&directory.0).unwrap(), registered.id.clone()).unwrap();
        worker.advance_elapsed(Duration::from_secs(10)).unwrap();
        drop(worker);

        thread::sleep(Duration::from_millis(5));
        let pending = Store::open_at(&directory.0)
            .unwrap()
            .get_desktop(&registered.id)
            .unwrap();
        assert_eq!(pending.state.bars.task.position, 6_000);
        assert_eq!(pending.state.current_task, initial_task);

        let mut restarted =
            Worker::start(Store::open_at(&directory.0).unwrap(), registered.id.clone()).unwrap();
        restarted.advance_elapsed(Duration::ZERO).unwrap();
        let completed = restarted.store.get_desktop(&registered.id).unwrap();
        assert_eq!(completed.state.bars.task.position, 0);
        assert_ne!(completed.state.current_task, initial_task);
    }

    #[test]
    fn desktop_worker_commits_before_serialized_report_and_does_not_replay() {
        let directory = TestDirectory::new("desktop-worker-report-order");
        let mut store = Store::open_at(&directory.0).unwrap();
        let registered = store
            .register_desktop(&desktop_level_report_fixture(), &mut Numbers(0x1357_9bdf))
            .unwrap();
        let online_action = store.acquire_online_action_lock(&registered.id).unwrap();
        let path = directory.0.clone();
        let id = registered.id.clone();
        let (sender, receiver) = mpsc::channel();
        let reporting = thread::spawn(move || {
            let mut worker = Worker::start(Store::open_at(path).unwrap(), id).unwrap();
            let result = worker.advance_elapsed(Duration::ZERO);
            sender.send(result).unwrap();
        });

        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            let persisted = Store::open_at(&directory.0)
                .unwrap()
                .get_desktop(&registered.id)
                .unwrap();
            if persisted.identity.level == 3 {
                break;
            }
            assert!(
                Instant::now() < deadline,
                "desktop callback was not persisted before report serialization"
            );
            thread::sleep(Duration::from_millis(10));
        }
        assert!(receiver.recv_timeout(Duration::from_millis(100)).is_err());

        store
            .replace_profile(
                &registered.id,
                &OnlineProfile {
                    motto: "Serialized before report".to_owned(),
                    guild: "Desktop Gnomes".to_owned(),
                },
            )
            .unwrap();
        drop(online_action);
        receiver
            .recv_timeout(Duration::from_secs(1))
            .unwrap()
            .unwrap();
        reporting.join().unwrap();

        let persisted = Store::open_at(&directory.0)
            .unwrap()
            .get_desktop(&registered.id)
            .unwrap();
        assert_eq!(persisted.identity.level, 3);
        assert_eq!(
            persisted
                .import_metadata
                .measured_since_import
                .tasks_completed,
            1
        );
        assert_eq!(
            persisted.import_metadata.advancement_provenance,
            DesktopAdvancementProvenance::LocalOnly
        );
        assert_eq!(persisted.state.profile.motto, "Serialized before report");
        assert_eq!(persisted.state.profile.guild, "Desktop Gnomes");

        let online_action = store.acquire_online_action_lock(&registered.id).unwrap();
        let path = directory.0.clone();
        let id = registered.id.clone();
        let (sender, receiver) = mpsc::channel();
        let resumed = thread::spawn(move || {
            let mut worker = Worker::start(Store::open_at(path).unwrap(), id).unwrap();
            sender
                .send(worker.advance_elapsed(Duration::from_millis(100)))
                .unwrap();
        });
        receiver
            .recv_timeout(Duration::from_secs(1))
            .expect("restart replayed an already-consumed report")
            .unwrap();
        drop(online_action);
        resumed.join().unwrap();
    }

    #[test]
    fn interrupted_desktop_callback_keeps_complete_checkpoint_and_emits_no_report() {
        let directory = TestDirectory::new("desktop-worker-interrupted-update");
        let mut store = Store::open_at(&directory.0).unwrap();
        let registered = store
            .register_desktop(&desktop_level_report_fixture(), &mut Numbers(0x1357_9bdf))
            .unwrap();
        let original = store.get_desktop(&registered.id).unwrap();
        let online_action = store.acquire_online_action_lock(&registered.id).unwrap();
        let mut worker =
            Worker::start(Store::open_at(&directory.0).unwrap(), registered.id.clone()).unwrap();
        worker.store.inject_next_update_failure();

        assert!(matches!(
            worker.advance_elapsed(Duration::ZERO),
            Err(WorkerError::Storage(StorageError::InjectedFailure))
        ));
        drop(online_action);
        let restored = Store::open_at(&directory.0)
            .unwrap()
            .get_desktop(&registered.id)
            .unwrap();
        assert_eq!(restored.state, original.state);
        assert_eq!(restored.compatibility.random, original.compatibility.random);
        assert_eq!(restored.import_metadata, original.import_metadata);
    }

    #[test]
    fn desktop_local_only_mark_requires_online_origin_and_actual_advancement() {
        let directory = TestDirectory::new("desktop-local-only-boundary");
        let mut store = Store::open_at(&directory.0).unwrap();
        let online = store
            .register_desktop(&desktop_fixture(), &mut Numbers(1))
            .unwrap();
        let mut offline_save = desktop_fixture();
        offline_save.private = DesktopValidatedPrivateMetadata {
            passkey: 0,
            realm: String::new(),
            endpoint: String::new(),
            account: String::new(),
            password: String::new(),
        };
        let offline = store
            .register_desktop(&offline_save, &mut Numbers(2))
            .unwrap();

        let mut online_worker =
            Worker::start(Store::open_at(&directory.0).unwrap(), online.id.clone()).unwrap();
        online_worker.advance_elapsed(Duration::ZERO).unwrap();
        assert_eq!(
            online_worker
                .store
                .get_desktop(&online.id)
                .unwrap()
                .import_metadata
                .advancement_provenance,
            DesktopAdvancementProvenance::Unadvanced
        );
        online_worker
            .advance_elapsed(Duration::from_millis(1))
            .unwrap();
        assert_eq!(
            online_worker
                .store
                .get_desktop(&online.id)
                .unwrap()
                .import_metadata
                .advancement_provenance,
            DesktopAdvancementProvenance::LocalOnly
        );
        drop(online_worker);

        let mut offline_worker =
            Worker::start(Store::open_at(&directory.0).unwrap(), offline.id.clone()).unwrap();
        offline_worker
            .advance_elapsed(Duration::from_millis(100))
            .unwrap();
        assert_eq!(
            offline_worker
                .store
                .get_desktop(&offline.id)
                .unwrap()
                .import_metadata
                .advancement_provenance,
            DesktopAdvancementProvenance::Unadvanced
        );
    }

    #[test]
    fn evidenced_spoltog_desktop_scope_stays_online_and_uses_desktop_delivery() {
        let directory = TestDirectory::new("desktop-evidenced-online");
        let mut store = Store::open_at(&directory.0).unwrap();
        let registered = store
            .register_desktop(&evidenced_spoltog_desktop_fixture(), &mut Numbers(7))
            .unwrap();
        let inspection = store.managed_inspection(&registered.id).unwrap();
        assert!(
            inspection
                .compatibility
                .online_eligibility
                .iter()
                .all(|eligibility| eligibility.decision == DesktopEligibilityDecision::Eligible)
        );

        let transport = RejectNetworkTransport;
        assert_eq!(
            crate::reporting::submit(&store, &registered.id, &transport)
                .unwrap()
                .outcome,
            DeliveryOutcome::DeliveryFailed
        );
        assert_eq!(
            crate::reporting::set_motto(&mut store, &registered.id, "Evidenced motto", &transport,)
                .unwrap()
                .outcome,
            DeliveryOutcome::DeliveryFailed
        );
        assert_eq!(
            store.profile(&registered.id).unwrap().motto,
            "Evidenced motto"
        );
        assert_eq!(
            crate::reporting::set_guild(&mut store, &registered.id, "QoD", &transport)
                .unwrap()
                .outcome,
            GuildOutcome::Indeterminate
        );

        let state = store.get_desktop(&registered.id).unwrap().state;
        assert_eq!(
            crate::reporting::submit_desktop_event(
                &store,
                &registered.id,
                &DesktopReportSnapshot {
                    trigger: DesktopReportTrigger::Level,
                    state,
                },
                &transport,
            )
            .unwrap()
            .outcome,
            DeliveryOutcome::DeliveryFailed
        );

        let mut worker =
            Worker::start(Store::open_at(&directory.0).unwrap(), registered.id.clone()).unwrap();
        worker.advance_elapsed(Duration::from_millis(1)).unwrap();
        assert_eq!(
            worker
                .store
                .get_desktop(&registered.id)
                .unwrap()
                .import_metadata
                .advancement_provenance,
            DesktopAdvancementProvenance::Unadvanced
        );
    }

    #[test]
    fn bundled_placeholder_coverage_enables_all_operations_and_never_reconnects_local_forks() {
        let directory = TestDirectory::new("pemptus-placeholder-production");
        let mut source = pemptus_placeholder_fixture();
        let level = desktop_level_report_fixture();
        source.current_task = level.current_task;
        source.activity = level.activity;
        source.queue = level.queue;
        source.bars = level.bars;
        let mut store = Store::open_at(&directory.0).unwrap();
        let fresh = store.register_desktop(&source, &mut Numbers(7)).unwrap();
        let forked = store.register_desktop(&source, &mut Numbers(8)).unwrap();
        crate::desktop_evidence::with_synthetic_desktop_evidence(vec![], || {
            let mut worker = Worker::start_with_transport(
                Store::open_at(&directory.0).unwrap(),
                forked.id.clone(),
                RejectNetworkTransport,
            )
            .unwrap();
            worker.advance_elapsed(Duration::from_millis(1)).unwrap();
        });
        let gates = store
            .managed_inspection(&fresh.id)
            .unwrap()
            .compatibility
            .online_eligibility;
        assert_eq!(gates, save::inspect_desktop(&source).online_eligibility);
        for gate in gates {
            assert_eq!(gate.decision, DesktopEligibilityDecision::Eligible);
        }
        assert!(
            store
                .managed_inspection(&forked.id)
                .unwrap()
                .compatibility
                .online_eligibility
                .iter()
                .all(|gate| gate.decision
                    == DesktopEligibilityDecision::Ineligible(
                        DesktopIneligibilityReason::FreshOfficialClientImportRequired
                    ))
        );
        assert!(crate::reporting::submit(&store, &forked.id, &RejectNetworkTransport).is_err());
        assert!(crate::reporting::set_motto(
            &mut store, &forked.id, "Blocked", &RejectNetworkTransport,
        ).is_err());
        assert!(crate::reporting::set_guild(
            &mut store, &forked.id, "Blocked", &RejectNetworkTransport,
        ).is_err());
        let calls = Arc::new(Mutex::new(vec![]));
        let transport = PemptusTransport {
            path: directory.0.clone(),
            id: fresh.id.clone(),
            calls: calls.clone(),
            fail: false,
        };
        let mut worker = Worker::start_with_transport(
            Store::open_at(&directory.0).unwrap(),
            fresh.id.clone(),
            transport,
        )
        .unwrap();
        worker.advance_elapsed(Duration::ZERO).unwrap();
        worker.advance_elapsed(Duration::from_millis(1)).unwrap();
        assert_eq!(calls.lock().unwrap().as_slice(), ["l"]);
        let state = worker.store.get_desktop(&fresh.id).unwrap();
        assert_eq!(
            state.import_metadata.provenance.adaptations,
            [crate::compatibility::DesktopAdaptation::LegacyQuestPlaceholder]
        );
        assert_eq!(
            state.import_metadata.advancement_provenance,
            DesktopAdvancementProvenance::Unadvanced
        );
        assert!(!matches!(
            state.state.quest,
            DesktopQuestMarker::LegacyPlaceholder { .. }
        ));
        assert_eq!(
            worker
                .store
                .managed_inspection(&fresh.id)
                .unwrap()
                .compatibility
                .online_eligibility,
            save::inspect_desktop(&source).online_eligibility
        );
    }

    #[test]
    fn bundled_pemptus_coverage_supports_existing_imports_without_reimport_or_local_fork() {
        for path in 0..3 {
            let directory = TestDirectory::new(&format!("pemptus-production-supported-{path}"));
            let mut store = Store::open_at(&directory.0).unwrap();
            let mut save = if path == 2 {
                pemptus_placeholder_fixture()
            } else {
                pemptus_fixture()
            };
            save.adaptations.spelling_patch_applied = path == 1;
            let level = desktop_level_report_fixture();
            save.current_task = level.current_task;
            save.activity = level.activity;
            save.queue = level.queue;
            save.bars = level.bars;
            let registered = store.register_desktop(&save, &mut Numbers(7)).unwrap();
            let original_metadata = store.get_desktop(&registered.id).unwrap().import_metadata;
            crate::desktop_evidence::with_synthetic_desktop_evidence(vec![], || {
                assert!(
                    store
                        .managed_inspection(&registered.id)
                        .unwrap()
                        .compatibility
                        .online_eligibility
                        .iter()
                        .filter(|gate| gate.operation != DesktopOnlineOperation::Guild)
                        .all(|gate| gate.decision != DesktopEligibilityDecision::Eligible)
                );
            });
            let listed = store.list_managed().unwrap();
            let compatibility = &listed
                .iter()
                .find(|value| value.id == registered.id)
                .unwrap()
                .compatibility;
            assert!(compatibility.notice.is_none());
            assert_eq!(
                compatibility
                    .online_eligibility
                    .iter()
                    .map(|gate| format!("{:?}: {:?}", gate.operation, gate.decision))
                    .collect::<Vec<_>>(),
                [
                    "AutomaticLevel: Eligible",
                    "AutomaticAct: Eligible",
                    "ManualBrag: Eligible",
                    "Motto: Eligible",
                    "Guild: Eligible",
                ],
            );
            let calls = Arc::new(Mutex::new(vec![]));
            let transport = PemptusTransport {
                path: directory.0.clone(),
                id: registered.id.clone(),
                calls: calls.clone(),
                fail: false,
            };
            assert_eq!(
                save::inspect_desktop(&save).online_eligibility,
                store
                    .managed_inspection(&registered.id)
                    .unwrap()
                    .compatibility
                    .online_eligibility,
            );
            assert!(
                store
                    .managed_inspection(&registered.id)
                    .unwrap()
                    .compatibility
                    .online_eligibility
                    .iter()
                    .all(|gate| gate.decision == DesktopEligibilityDecision::Eligible)
            );
            assert_eq!(
                crate::reporting::submit(&store, &registered.id, &transport)
                    .unwrap()
                    .outcome,
                DeliveryOutcome::Delivered,
            );
            assert_eq!(
                crate::reporting::set_motto(
                    &mut store,
                    &registered.id,
                    "Supported motto",
                    &transport
                )
                .unwrap()
                .outcome,
                DeliveryOutcome::Delivered,
            );
            assert_eq!(
                crate::reporting::set_motto(&mut store, &registered.id, "", &transport)
                    .unwrap()
                    .outcome,
                DeliveryOutcome::Delivered,
            );
            for designation in ["Guild A", "Guild B", ""] {
                assert_eq!(
                    crate::reporting::set_guild(
                        &mut store,
                        &registered.id,
                        designation,
                        &transport,
                    )
                    .unwrap()
                    .outcome,
                    GuildOutcome::Accepted
                );
                assert_eq!(store.profile(&registered.id).unwrap().guild, designation);
            }
            assert_eq!(calls.lock().unwrap().len(), 9);
            let mut worker = Worker::start_with_transport(
                Store::open_at(&directory.0).unwrap(),
                registered.id.clone(),
                transport,
            )
            .unwrap();
            worker.advance_elapsed(Duration::ZERO).unwrap();
            worker.advance_elapsed(Duration::from_millis(1)).unwrap();
            drop(worker);
            let reopened = Store::open_at(&directory.0).unwrap();
            assert_eq!(
                reopened
                    .get_desktop(&registered.id)
                    .unwrap()
                    .import_metadata
                    .advancement_provenance,
                DesktopAdvancementProvenance::Unadvanced,
            );
            assert!(
                reopened.list_managed().unwrap()[0]
                    .compatibility
                    .notice
                    .is_none()
            );
            assert_eq!(
                reopened
                    .get_desktop(&registered.id)
                    .unwrap()
                    .import_metadata
                    .provenance,
                original_metadata.provenance
            );
            assert!(
                reopened
                    .managed_inspection(&registered.id)
                    .unwrap()
                    .compatibility
                    .online_eligibility
                    .iter()
                    .all(|gate| gate.decision == DesktopEligibilityDecision::Eligible)
            );
            assert_eq!(calls.lock().unwrap().last().unwrap(), "l");
            assert_eq!(calls.lock().unwrap().len(), 10);
        }
    }

    #[test]
    fn pemptus_guild_actions_confirm_public_state_once_and_preserve_uncertain_membership() {
        use crate::desktop_transport::{
            DesktopHttpResponse, DesktopTransportCredentials, DesktopTransportError,
            VerifiedDesktopEndpoint,
        };
        struct ObservedGuildTransport {
            native: PemptusTransport,
            observation: Result<Option<String>, DesktopTransportError>,
        }
        impl GuildTransport for ObservedGuildTransport {
            fn guild(&self, _: Url, _: &str, _: &str, _: &GuildResponseRules) -> GuildOutcome {
                panic!("Pemptus used browser guild transport")
            }

            fn guild_desktop(
                &self,
                target: &VerifiedDesktopEndpoint,
                query: &str,
                credentials: &DesktopTransportCredentials,
            ) -> Result<DesktopHttpResponse, DesktopTransportError> {
                self.native.guild_desktop(target, query, credentials)
            }

            fn desktop_public_guild(
                &self,
                target: &VerifiedDesktopEndpoint,
                name: &str,
            ) -> Result<Option<String>, DesktopTransportError> {
                assert_eq!(target.realm(), "Pemptus");
                let stored = Store::open_at(&self.native.path)
                    .unwrap()
                    .get_desktop(&self.native.id)
                    .unwrap();
                assert_eq!(name, stored.identity.name);
                self.native
                    .calls
                    .lock()
                    .unwrap()
                    .push(format!("public:{name}"));
                self.observation
                    .as_ref()
                    .map(Clone::clone)
                    .map_err(|_| DesktopTransportError::DeliveryFailed)
            }
        }
        let directory = TestDirectory::new("pemptus-public-confirmation");
        let mut store = Store::open_at(&directory.0).unwrap();
        let registered = store
            .register_desktop(&pemptus_fixture(), &mut Numbers(7))
            .unwrap();
        for (designation, delivery_failed, observation, expected) in [
            (
                "Guild A",
                false,
                Ok(Some("guild a".to_owned())),
                GuildOutcome::Accepted,
            ),
            (
                "Guild B",
                true,
                Ok(Some("GUILD B".to_owned())),
                GuildOutcome::Accepted,
            ),
            ("", false, Ok(None), GuildOutcome::Accepted),
            (
                "Invalid",
                false,
                Ok(Some("Old Guild".to_owned())),
                GuildOutcome::Indeterminate,
            ),
            ("Guild A", false, Ok(None), GuildOutcome::Indeterminate),
            (
                "Guild A",
                false,
                Err(DesktopTransportError::DeliveryFailed),
                GuildOutcome::Indeterminate,
            ),
            (
                "",
                false,
                Ok(Some("Still Guild".to_owned())),
                GuildOutcome::Indeterminate,
            ),
        ] {
            let previous = store.profile(&registered.id).unwrap().guild;
            let confirmed = observation
                .as_ref()
                .ok()
                .cloned()
                .flatten()
                .unwrap_or_default();
            let calls = Arc::new(Mutex::new(vec![]));
            let transport = ObservedGuildTransport {
                native: PemptusTransport {
                    path: directory.0.clone(),
                    id: registered.id.clone(),
                    calls: calls.clone(),
                    fail: delivery_failed,
                },
                observation,
            };
            let result =
                crate::reporting::set_guild(&mut store, &registered.id, designation, &transport)
                    .unwrap();
            assert_eq!(result.outcome, expected);
            assert_eq!(
                store.profile(&registered.id).unwrap().guild,
                if expected == GuildOutcome::Accepted {
                    confirmed
                } else {
                    previous
                }
            );
            assert_eq!(
                calls.lock().unwrap().as_slice(),
                [
                    format!("guild:{designation}"),
                    format!("public:{}", result.identity.name),
                ]
            );
        }
    }

    #[test]
    fn pemptus_inspection_runtime_and_explicit_profiles_share_independent_gates() {
        use crate::desktop_evidence::{
            DesktopGuildFingerprints, synthetic_pemptus_evidence, with_synthetic_desktop_evidence,
        };
        let directory = TestDirectory::new("pemptus-profiles");
        let mut store = Store::open_at(&directory.0).unwrap();
        let save = pemptus_fixture();
        let registered = store.register_desktop(&save, &mut Numbers(7)).unwrap();
        let calls = Arc::new(Mutex::new(vec![]));
        let transport = PemptusTransport {
            path: directory.0.clone(),
            id: registered.id.clone(),
            calls: calls.clone(),
            fail: false,
        };
        with_synthetic_desktop_evidence(vec![], || {
            assert!(crate::reporting::submit(&store, &registered.id, &transport).is_err());
        });
        assert!(calls.lock().unwrap().is_empty());
        let values = crate::desktop_fingerprint::DesktopGuildFingerprintValues {
            character_name: "Desktop Hero",
            account: "",
            password: "",
            authorization: "",
            passkey: "4242",
            prior_guild: "Synthetic Guild",
            submitted_guild: "Guild A",
        };
        let records = vec![
            synthetic_pemptus_evidence(DesktopOnlineOperation::ManualBrag, None),
            synthetic_pemptus_evidence(DesktopOnlineOperation::Motto, None),
            synthetic_pemptus_evidence(
                DesktopOnlineOperation::Guild,
                Some(DesktopGuildFingerprints {
                    normalization: crate::desktop_fingerprint::DESKTOP_RESPONSE_FINGERPRINT_VERSION
                        .to_owned(),
                    join: values.fingerprint(b"joined"),
                    change: Some(values.fingerprint(b"changed")),
                    leave: values.fingerprint(b"left"),
                    rejected: values.fingerprint(b"rejected"),
                }),
            ),
        ];
        with_synthetic_desktop_evidence(records, || {
            assert_eq!(
                save::inspect_desktop(&save).online_eligibility,
                store
                    .managed_inspection(&registered.id)
                    .unwrap()
                    .compatibility
                    .online_eligibility
            );
            assert_eq!(
                crate::reporting::submit(&store, &registered.id, &transport)
                    .unwrap()
                    .outcome,
                DeliveryOutcome::Delivered
            );
            for motto in ["Pemptus motto", ""] {
                assert_eq!(
                    crate::reporting::set_motto(&mut store, &registered.id, motto, &transport)
                        .unwrap()
                        .outcome,
                    DeliveryOutcome::Delivered
                );
                assert_eq!(store.profile(&registered.id).unwrap().motto, motto);
            }
            for designation in ["Guild A", "Guild B"] {
                assert_eq!(
                    crate::reporting::set_guild(
                        &mut store,
                        &registered.id,
                        designation,
                        &transport
                    )
                    .unwrap()
                    .outcome,
                    GuildOutcome::Accepted
                );
                assert_eq!(store.profile(&registered.id).unwrap().guild, designation);
            }
            assert_eq!(
                crate::reporting::set_guild(&mut store, &registered.id, "Invalid", &transport)
                    .unwrap()
                    .outcome,
                GuildOutcome::Indeterminate
            );
            assert_eq!(store.profile(&registered.id).unwrap().guild, "Guild B");
            assert_eq!(
                crate::reporting::set_guild(&mut store, &registered.id, "", &transport)
                    .unwrap()
                    .outcome,
                GuildOutcome::Accepted
            );
            let before = calls.lock().unwrap().len();
            assert!(
                crate::reporting::set_motto(&mut store, &registered.id, "\u{e9}", &transport)
                    .is_err()
            );
            assert_eq!(calls.lock().unwrap().len(), before);
            assert_eq!(store.profile(&registered.id).unwrap().motto, "");
        });
        assert_eq!(calls.lock().unwrap().len(), 11);
        let mut non_ascii = pemptus_fixture();
        non_ascii.profile.motto = "\u{e9}".to_owned();
        let other = store.register_desktop(&non_ascii, &mut Numbers(8)).unwrap();
        assert_eq!(
            save::inspect_desktop(&non_ascii).online_eligibility,
            store
                .managed_inspection(&other.id)
                .unwrap()
                .compatibility
                .online_eligibility
        );
    }

    #[test]
    fn pemptus_placeholder_inspection_runtime_and_reporting_select_original_path() {
        use crate::desktop_evidence::{
            synthetic_pemptus_evidence, synthetic_pemptus_evidence_for_import,
            with_synthetic_desktop_evidence,
        };
        let path = vec![crate::compatibility::DesktopAdaptation::LegacyQuestPlaceholder];
        let directory = TestDirectory::new("pemptus-placeholder");
        let mut save = pemptus_placeholder_fixture();
        let level = desktop_level_report_fixture();
        save.current_task = level.current_task;
        save.activity = level.activity;
        save.queue = level.queue;
        save.bars = level.bars;
        let mut store = Store::open_at(&directory.0).unwrap();
        let registered = store
            .register_desktop(&save, &mut Numbers(0x1357_9bdf))
            .unwrap();
        let before = store.get_desktop(&registered.id).unwrap().import_metadata;
        let mut records: Vec<_> = [
            DesktopOnlineOperation::AutomaticLevel,
            DesktopOnlineOperation::AutomaticAct,
        ]
        .into_iter()
        .map(|operation| synthetic_pemptus_evidence_for_import(operation, None, path.clone()))
        .collect();
        records.extend(
            [
                DesktopOnlineOperation::ManualBrag,
                DesktopOnlineOperation::Motto,
            ]
            .into_iter()
            .map(|operation| synthetic_pemptus_evidence(operation, None)),
        );
        with_synthetic_desktop_evidence(records, || {
            let inspection = save::inspect_desktop(&save);
            assert_eq!(
                inspection.online_eligibility,
                store
                    .managed_inspection(&registered.id)
                    .unwrap()
                    .compatibility
                    .online_eligibility
            );
            for value in inspection.online_eligibility {
                assert_eq!(value.decision, DesktopEligibilityDecision::Eligible);
            }
            assert_eq!(
                store.profile(&registered.id).unwrap().motto,
                save.profile.motto
            );
            let unadapted = store
                .register_desktop(&pemptus_fixture(), &mut Numbers(8))
                .unwrap();
            let gates = store
                .managed_inspection(&unadapted.id)
                .unwrap()
                .compatibility
                .online_eligibility;
            assert!(
                gates
                    .iter()
                    .filter(|value| matches!(
                        value.operation,
                        DesktopOnlineOperation::AutomaticLevel
                            | DesktopOnlineOperation::AutomaticAct
                    ))
                    .all(|value| value.decision == DesktopEligibilityDecision::Eligible)
            );
            for spelling in [true, false] {
                let mut combined = save.clone();
                combined.adaptations.spelling_patch_applied = spelling;
                combined.adaptations.legacy_prologue_62 = !spelling;
                let registered = store.register_desktop(&combined, &mut Numbers(9)).unwrap();
                let gates = store
                    .managed_inspection(&registered.id)
                    .unwrap()
                    .compatibility
                    .online_eligibility;
                assert_eq!(gates, save::inspect_desktop(&combined).online_eligibility);
                assert!(
                    gates
                        .iter()
                        .all(|value| value.decision != DesktopEligibilityDecision::Eligible)
                );
                assert!(
                    crate::reporting::submit(&store, &registered.id, &RejectNetworkTransport)
                        .is_err()
                );
            }
            let calls = Arc::new(Mutex::new(vec![]));
            let transport = PemptusTransport {
                path: directory.0.clone(),
                id: registered.id.clone(),
                calls: calls.clone(),
                fail: false,
            };
            let mut worker = Worker::start_with_transport(
                Store::open_at(&directory.0).unwrap(),
                registered.id.clone(),
                transport,
            )
            .unwrap();
            worker.advance_elapsed(Duration::ZERO).unwrap();
            assert_eq!(calls.lock().unwrap().as_slice(), ["l"]);
            let persisted = worker.store.get_desktop(&registered.id).unwrap();
            let mut expected_metadata = before.clone();
            expected_metadata.measured_since_import.tasks_completed += 1;
            assert_eq!(persisted.import_metadata, expected_metadata);
            assert_eq!(persisted.import_metadata.provenance.adaptations, path);
            assert!(!matches!(
                persisted.state.quest,
                DesktopQuestMarker::LegacyPlaceholder { .. }
            ));
            assert_eq!(
                worker
                    .store
                    .managed_inspection(&registered.id)
                    .unwrap()
                    .compatibility
                    .online_eligibility,
                save::inspect_desktop(&save).online_eligibility
            );
            drop(worker);
            let reopened = Store::open_at(&directory.0).unwrap();
            assert_eq!(
                reopened
                    .get_desktop(&registered.id)
                    .unwrap()
                    .import_metadata,
                expected_metadata
            );
        });
        with_synthetic_desktop_evidence(vec![], || {
            assert!(
                store
                    .managed_inspection(&registered.id)
                    .unwrap()
                    .compatibility
                    .online_eligibility
                    .iter()
                    .filter(|value| value.operation != DesktopOnlineOperation::Guild)
                    .all(|value| value.decision != DesktopEligibilityDecision::Eligible)
            );
        });
    }

    #[test]
    fn pemptus_placeholder_missing_either_automatic_record_forks_permanently() {
        use crate::desktop_evidence::{
            synthetic_pemptus_evidence_for_import, with_synthetic_desktop_evidence,
        };
        let path = vec![crate::compatibility::DesktopAdaptation::LegacyQuestPlaceholder];
        for available in [
            DesktopOnlineOperation::AutomaticLevel,
            DesktopOnlineOperation::AutomaticAct,
        ] {
            let directory = TestDirectory::new("pemptus-placeholder-fork");
            let mut store = Store::open_at(&directory.0).unwrap();
            let source = pemptus_placeholder_fixture();
            let registered = store.register_desktop(&source, &mut Numbers(7)).unwrap();
            let untouched = store.register_desktop(&source, &mut Numbers(8)).unwrap();
            with_synthetic_desktop_evidence(
                vec![synthetic_pemptus_evidence_for_import(
                    available,
                    None,
                    path.clone(),
                )],
                || {
                    let mut worker = Worker::start_with_transport(
                        Store::open_at(&directory.0).unwrap(),
                        registered.id.clone(),
                        RejectNetworkTransport,
                    )
                    .unwrap();
                    worker.advance_elapsed(Duration::from_millis(1)).unwrap();
                    assert_eq!(
                        worker
                            .store
                            .get_desktop(&registered.id)
                            .unwrap()
                            .import_metadata
                            .advancement_provenance,
                        DesktopAdvancementProvenance::LocalOnly
                    );
                },
            );
            let records = [
                DesktopOnlineOperation::AutomaticLevel,
                DesktopOnlineOperation::AutomaticAct,
            ]
            .into_iter()
            .map(|operation| synthetic_pemptus_evidence_for_import(operation, None, path.clone()))
            .collect();
            with_synthetic_desktop_evidence(records, || {
                let reopened = Store::open_at(&directory.0).unwrap();
                assert!(
                    reopened
                        .managed_inspection(&registered.id)
                        .unwrap()
                        .compatibility
                        .online_eligibility
                        .iter()
                        .all(|value| value.decision
                            == DesktopEligibilityDecision::Ineligible(
                                DesktopIneligibilityReason::FreshOfficialClientImportRequired
                            ))
                );
                assert!(
                    reopened
                        .managed_inspection(&untouched.id)
                        .unwrap()
                        .compatibility
                        .online_eligibility
                        .iter()
                        .filter(|value| matches!(
                            value.operation,
                            DesktopOnlineOperation::AutomaticLevel
                                | DesktopOnlineOperation::AutomaticAct
                        ))
                        .all(|value| value.decision == DesktopEligibilityDecision::Eligible)
                );
                assert_eq!(
                    reopened
                        .get_desktop(&registered.id)
                        .unwrap()
                        .import_metadata
                        .provenance
                        .adaptations,
                    path
                );
                assert!(
                    crate::reporting::submit(&reopened, &registered.id, &RejectNetworkTransport)
                        .is_err()
                );
            });
        }
    }

    #[test]
    fn pemptus_auto_only_coverage_preserves_ordering_without_fork_retry_or_catch_up() {
        use crate::desktop_evidence::{
            synthetic_pemptus_evidence, with_synthetic_desktop_evidence,
        };
        let directory = TestDirectory::new("pemptus-worker");
        let mut save = pemptus_fixture();
        let level = desktop_level_report_fixture();
        save.current_task = level.current_task;
        save.activity = level.activity;
        save.queue = level.queue;
        save.bars = level.bars;
        let mut store = Store::open_at(&directory.0).unwrap();
        let registered = store
            .register_desktop(&save, &mut Numbers(0x1357_9bdf))
            .unwrap();
        let calls = Arc::new(Mutex::new(vec![]));
        let records = [
            DesktopOnlineOperation::AutomaticLevel,
            DesktopOnlineOperation::AutomaticAct,
        ]
        .into_iter()
        .map(|operation| synthetic_pemptus_evidence(operation, None))
        .collect();
        with_synthetic_desktop_evidence(records, || {
            let transport = PemptusTransport {
                path: directory.0.clone(),
                id: registered.id.clone(),
                calls: calls.clone(),
                fail: true,
            };
            let mut worker = Worker::start_with_transport(
                Store::open_at(&directory.0).unwrap(),
                registered.id.clone(),
                transport,
            )
            .unwrap();
            worker.advance_elapsed(Duration::ZERO).unwrap();
            assert_eq!(calls.lock().unwrap().as_slice(), ["l"]);
            assert_eq!(
                worker
                    .store
                    .get_desktop(&registered.id)
                    .unwrap()
                    .import_metadata
                    .advancement_provenance,
                DesktopAdvancementProvenance::Unadvanced
            );
            drop(worker);
            let transport = PemptusTransport {
                path: directory.0.clone(),
                id: registered.id.clone(),
                calls: calls.clone(),
                fail: true,
            };
            let mut resumed = Worker::start_with_transport(
                Store::open_at(&directory.0).unwrap(),
                registered.id.clone(),
                transport,
            )
            .unwrap();
            resumed.advance_elapsed(Duration::from_millis(100)).unwrap();
            assert_eq!(calls.lock().unwrap().as_slice(), ["l"]);
            assert_eq!(
                resumed
                    .store
                    .get_desktop(&registered.id)
                    .unwrap()
                    .state
                    .bars
                    .task
                    .position,
                100
            );
        });
    }

    #[test]
    fn pemptus_partial_progress_gate_makes_a_durable_irreversible_local_fork() {
        use crate::desktop_evidence::{
            synthetic_pemptus_evidence, with_synthetic_desktop_evidence,
        };
        let directory = TestDirectory::new("pemptus-local-fork");
        let mut store = Store::open_at(&directory.0).unwrap();
        let registered = store
            .register_desktop(&pemptus_fixture(), &mut Numbers(7))
            .unwrap();
        let untouched = store
            .register_desktop(&pemptus_fixture(), &mut Numbers(8))
            .unwrap();
        with_synthetic_desktop_evidence(
            vec![synthetic_pemptus_evidence(
                DesktopOnlineOperation::AutomaticLevel,
                None,
            )],
            || {
                let mut worker = Worker::start_with_transport(
                    Store::open_at(&directory.0).unwrap(),
                    registered.id.clone(),
                    RejectNetworkTransport,
                )
                .unwrap();
                worker.advance_elapsed(Duration::from_millis(1)).unwrap();
                assert_eq!(
                    worker
                        .store
                        .get_desktop(&registered.id)
                        .unwrap()
                        .import_metadata
                        .advancement_provenance,
                    DesktopAdvancementProvenance::LocalOnly
                );
            },
        );
        let records = [
            DesktopOnlineOperation::AutomaticLevel,
            DesktopOnlineOperation::AutomaticAct,
            DesktopOnlineOperation::ManualBrag,
            DesktopOnlineOperation::Motto,
        ]
        .into_iter()
        .map(|operation| synthetic_pemptus_evidence(operation, None))
        .collect();
        with_synthetic_desktop_evidence(records, || {
            let reopened = Store::open_at(&directory.0).unwrap();
            assert!(
                reopened
                    .managed_inspection(&registered.id)
                    .unwrap()
                    .compatibility
                    .online_eligibility
                    .iter()
                    .all(|value| value.decision
                        == DesktopEligibilityDecision::Ineligible(
                            DesktopIneligibilityReason::FreshOfficialClientImportRequired,
                        ))
            );
            assert_eq!(
                reopened
                    .managed_inspection(&untouched.id)
                    .unwrap()
                    .compatibility
                    .online_eligibility[0]
                    .decision,
                DesktopEligibilityDecision::Eligible
            );
            assert!(
                crate::reporting::submit(&reopened, &registered.id, &RejectNetworkTransport)
                    .is_err()
            );
        });
    }

    #[test]
    fn desktop_local_only_mark_is_monotonic_and_depends_only_on_progress_evidence() {
        let authentication = DesktopAuthentication {
            passkey: 42,
            realm: "Synthetic Realm".to_owned(),
            endpoint: "https://synthetic.invalid/".to_owned(),
            account: "synthetic-account".to_owned(),
            password: "synthetic-password".to_owned(),
        };
        assert!(should_mark_desktop_local_only(&authentication, true, false));
        assert!(!should_mark_desktop_local_only(&authentication, true, true));
        let unenrolled_authentication = DesktopAuthentication {
            passkey: 0,
            ..authentication.clone()
        };
        assert!(!should_mark_desktop_local_only(
            &unenrolled_authentication,
            true,
            false
        ));

        let directory = TestDirectory::new("desktop-local-only-monotonic");
        let mut store = Store::open_at(&directory.0).unwrap();
        let registered = store
            .register_desktop(&desktop_fixture(), &mut Numbers(3))
            .unwrap();
        let mut worker =
            Worker::start(Store::open_at(&directory.0).unwrap(), registered.id.clone()).unwrap();
        worker.advance_elapsed(Duration::from_millis(100)).unwrap();
        let first = worker.store.get_desktop(&registered.id).unwrap();
        assert_eq!(
            first.import_metadata.advancement_provenance,
            DesktopAdvancementProvenance::LocalOnly
        );

        let RandomContinuation::Desktop644(random) = first.compatibility.random else {
            panic!("desktop fixture lost desktop random continuation");
        };
        let checkpoint = DesktopCallbackCheckpoint {
            state: first.state,
            random,
        };
        worker
            .store
            .replace_desktop_checkpoint(
                &registered.id,
                &checkpoint,
                DesktopCallbackObservation {
                    credited_milliseconds: 0,
                    completion_dispatched: false,
                },
                false,
            )
            .unwrap();
        assert_eq!(
            worker
                .store
                .get_desktop(&registered.id)
                .unwrap()
                .import_metadata
                .advancement_provenance,
            DesktopAdvancementProvenance::LocalOnly
        );
    }

    #[test]
    fn local_only_desktop_record_rejects_every_online_operation_until_fresh_import() {
        let directory = TestDirectory::new("desktop-fresh-import-required");
        let mut store = Store::open_at(&directory.0).unwrap();
        let registered = store
            .register_desktop(&desktop_level_report_fixture(), &mut Numbers(0x1357_9bdf))
            .unwrap();
        let transport = RejectNetworkTransport;
        let mut worker = Worker::start_with_transport(
            Store::open_at(&directory.0).unwrap(),
            registered.id.clone(),
            RejectNetworkTransport,
        )
        .unwrap();
        worker.advance_elapsed(Duration::ZERO).unwrap();
        drop(worker);

        let assert_fresh_import = |result: Result<_, ReportingError>| {
            assert!(matches!(
                result,
                Err(ReportingError::Storage(
                    StorageError::FreshDesktopImportRequired
                ))
            ));
        };
        assert_fresh_import(crate::reporting::submit(&store, &registered.id, &transport));
        assert_fresh_import(crate::reporting::set_motto(
            &mut store,
            &registered.id,
            "Replacement motto",
            &transport,
        ));
        assert!(matches!(
            crate::reporting::set_guild(
                &mut store,
                &registered.id,
                "Replacement guild",
                &transport,
            ),
            Err(ReportingError::Storage(
                StorageError::FreshDesktopImportRequired
            ))
        ));
        let persisted = store.get_desktop(&registered.id).unwrap();
        assert_eq!(persisted.state.profile.motto, "Synthetic motto");
        assert_eq!(persisted.state.profile.guild, "Synthetic Guild");
        assert!(matches!(
            crate::reporting::submit_desktop_event(
                &store,
                &registered.id,
                &DesktopReportSnapshot {
                    trigger: DesktopReportTrigger::Level,
                    state: persisted.state.clone(),
                },
                &transport,
            ),
            Err(ReportingError::Storage(
                StorageError::FreshDesktopImportRequired
            ))
        ));

        store
            .connection
            .execute(
                "UPDATE desktop_private
                 SET passkey = 9999, realm = 'Replacement Realm',
                     endpoint = 'https://replacement.invalid/',
                     account = 'replacement-account', password = 'replacement-password'
                 WHERE character_id = ?1",
                [registered.id.to_string()],
            )
            .unwrap();
        assert_fresh_import(crate::reporting::submit(&store, &registered.id, &transport));
        assert_eq!(
            store
                .get_desktop(&registered.id)
                .unwrap()
                .import_metadata
                .advancement_provenance,
            DesktopAdvancementProvenance::LocalOnly
        );

        let fresh = store
            .register_desktop(&desktop_fixture(), &mut Numbers(0x2468_ace0))
            .unwrap();
        assert_eq!(
            fresh.import_metadata.advancement_provenance,
            DesktopAdvancementProvenance::Unadvanced
        );
        let assert_outside_evidenced_realm = |result: Result<_, ReportingError>| {
            assert!(matches!(
                result,
                Err(ReportingError::Storage(
                    StorageError::DesktopOnlineIneligible(
                        DesktopIneligibilityReason::RealmMismatch
                    )
                ))
            ));
        };
        assert_outside_evidenced_realm(crate::reporting::submit(&store, &fresh.id, &transport));
        assert_outside_evidenced_realm(crate::reporting::set_motto(
            &mut store,
            &fresh.id,
            "Still local",
            &transport,
        ));
        assert!(matches!(
            crate::reporting::set_guild(&mut store, &fresh.id, "Still local", &transport),
            Err(ReportingError::Storage(
                StorageError::DesktopOnlineIneligible(DesktopIneligibilityReason::RealmMismatch)
            ))
        ));
        assert!(matches!(
            crate::reporting::submit_desktop_event(
                &store,
                &fresh.id,
                &DesktopReportSnapshot {
                    trigger: DesktopReportTrigger::Act,
                    state: fresh.state.clone(),
                },
                &transport,
            ),
            Err(ReportingError::Storage(
                StorageError::DesktopOnlineIneligible(DesktopIneligibilityReason::RealmMismatch)
            ))
        ));
        let persisted = store.get_desktop(&fresh.id).unwrap();
        assert_eq!(persisted.state.profile.motto, "Synthetic motto");
        assert_eq!(persisted.state.profile.guild, "Synthetic Guild");
        assert!(matches!(
            crate::reporting::set_motto(&mut store, &fresh.id, "Gnomé", &transport),
            Err(ReportingError::Storage(
                StorageError::DesktopOnlineIneligible(
                    DesktopIneligibilityReason::UnsupportedEncoding
                )
            ))
        ));
    }

    #[test]
    fn desktop_worker_rejects_profile_random_mismatch_without_browser_fallback() {
        let directory = TestDirectory::new("desktop-worker-profile-mismatch");
        let mut store = Store::open_at(&directory.0).unwrap();
        let registered = store
            .register_desktop(&desktop_fixture(), &mut Numbers(0xf00d_cafe))
            .unwrap();
        let browser_random =
            serde_json::to_string(&RandomContinuation::Browser(fixture_character().seed)).unwrap();
        store
            .connection
            .execute(
                "UPDATE characters SET random_continuation = ?1 WHERE id = ?2",
                params![browser_random, registered.id.to_string()],
            )
            .unwrap();
        let error =
            match Worker::start(Store::open_at(&directory.0).unwrap(), registered.id.clone()) {
                Ok(_) => panic!("mismatched desktop continuation unexpectedly started"),
                Err(error) => error,
            };
        assert!(
            matches!(
                error,
                WorkerError::Storage(StorageError::InvalidCompatibilityState(
                    "profile and random continuation do not match"
                ))
            ),
            "{error:?}"
        );
        assert!(matches!(
            Store::open_at(&directory.0)
                .unwrap()
                .get(&registered.id),
            Err(StorageError::NotFound(id)) if id == registered.id
        ));
    }

    #[test]
    fn browser_worker_rejects_profile_random_mismatch_without_desktop_fallback() {
        let directory = TestDirectory::new("browser-worker-profile-mismatch");
        let mut store = Store::open_at(&directory.0).unwrap();
        let registered = store.register(&fixture_character()).unwrap();
        let desktop_random = serde_json::to_string(&RandomContinuation::Desktop644(
            crate::compatibility::DesktopRandomState(0xf00d_cafe),
        ))
        .unwrap();
        store
            .connection
            .execute(
                "UPDATE characters SET random_continuation = ?1 WHERE id = ?2",
                params![desktop_random, registered.id.to_string()],
            )
            .unwrap();

        assert!(matches!(
            Worker::start(Store::open_at(&directory.0).unwrap(), registered.id),
            Err(WorkerError::Storage(
                StorageError::InvalidCompatibilityState(
                    "profile and random continuation do not match"
                )
            ))
        ));
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
