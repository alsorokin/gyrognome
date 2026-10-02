use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use base64::{Engine, engine::general_purpose::STANDARD};
use ring::digest::{SHA256, digest};
use serde::{Deserialize, Serialize};
use signal_hook::consts::{SIGINT, SIGTERM};
use thiserror::Error;
use url::Url;

use crate::{
    compatibility::{
        CompatibilityProfile, DesktopCanonicalState, DesktopImportMetadata, DesktopRandomState,
    },
    desktop_callback::DesktopCallbackCheckpoint,
    desktop_evidence::DESKTOP_ONLINE_IMPLEMENTATION_ID,
    desktop_fingerprint::{
        DESKTOP_RESPONSE_FINGERPRINT_VERSION, DesktopGuildFingerprintValues,
        normalized_response_fingerprint,
    },
    desktop_protocol::{
        DesktopAccountAuthentication, DesktopReportOperation, guild_request, report,
        report_for_snapshot,
    },
    desktop_rules::{CONFIG_DFM_SHA256, MAIN_PAS_SHA256, SOURCE_COMMIT, SOURCE_TAG},
    desktop_save::DesktopValidatedSave,
    desktop_simulation::{DesktopReportTrigger, SourceDerivedDesktopHooks},
    newguy::{OsRandom, RandomSource},
    save::{ImportedSave, import_supported_file},
};

const FORMAT: &str = "gyrognome-desktop-live-experiment/v2";
const REALM: &str = "Spoltog";
const SAVED_ENDPOINT: &str = "http://progressquest.com/spoltog.php?";
const VERIFIED_ENDPOINT: &str = "https://progressquest.com/spoltog.php";
const CALLBACK_INTERVAL: Duration = Duration::from_millis(100);
const PUBLIC_POLL_INTERVAL: Duration = Duration::from_secs(5);
const MAX_RESPONSE_BYTES: u64 = 16 * 1024;
const MIN_ACTIVE_SECONDS: u64 = 21_600;
const MAX_ACTIVE_SECONDS: u64 = 28_800;
const MAX_MOTTO_BYTES: usize = 255;
const MAX_GUILD_BYTES: usize = 255;
const MAX_DISTINCT_MOTTO_ATTEMPTS: u8 = 2;
const MAX_HANDOFF_PLOTS: usize = 2;

mod scoped;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum DesktopLiveStage {
    Legacy,
    Immediate,
    Progression,
}

#[derive(Debug)]
pub struct DesktopLiveExperimentOptions {
    pub realm: String,
    pub stage: DesktopLiveStage,
    pub operations: Vec<crate::desktop_eligibility::DesktopOnlineOperation>,
    pub max_mutation_attempts: Option<usize>,
    pub allow_existing_disposable: bool,
    pub allow_quest_placeholder: bool,
    pub initial_guild_leave: bool,
    pub preparation_reconciliation_seconds: Option<u64>,
    pub confirm_preparation_reconciliation: bool,
    pub operator_join_reconciliation_seconds: Option<u64>,
    pub confirm_operator_accepted_join: bool,
    pub confirm_no_competing_join: bool,
    pub manual_motto: String,
    pub guild_change_designation: Option<String>,
    pub confirm_disposable: bool,
    pub confirm_live_submission: bool,
    pub confirm_client_stopped: bool,
    pub confirm_cleanup: bool,
    pub validate_only: bool,
    pub immediate_only: bool,
    pub experiment_dir: PathBuf,
    pub evidence_path: PathBuf,
    pub control_motto: String,
    pub motto: String,
    pub guild_designation: String,
    pub max_active_seconds: u64,
    pub classification_poll_seconds: u64,
}

#[derive(Debug, Error)]
pub enum DesktopLiveExperimentError {
    #[error("refusing live experiment without all explicit confirmations")]
    MissingConfirmation,
    #[error("live experiment options are outside the approved bounded scope")]
    InvalidScope,
    #[error("the experiment directory must contain exactly one .pq save and at most one .bak")]
    InvalidExperimentDirectory,
    #[error("the disposable save is not a supported desktop save")]
    UnsupportedSave,
    #[error("the disposable save does not match the approved realm authentication contract")]
    ContractMismatch,
    #[error("the disposable save is not a fresh unadvanced character")]
    NotFresh,
    #[error("could not read or write private experiment state")]
    PrivateState,
    #[error("desktop progression failed")]
    Progression,
    #[error("desktop request construction failed")]
    Construction,
    #[error("desktop HTTPS delivery failed")]
    Delivery,
    #[error("public classification or profile verification was inconclusive")]
    Inconclusive,
    #[error("the disposable character was classified as a cheater")]
    Cheater,
    #[error("the experiment was interrupted; private checkpoint retained")]
    Interrupted,
    #[error("required level and act observations did not complete within the approved bound")]
    BoundExpired,
    #[error("cleanup could not be verified")]
    Cleanup,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PrivateCheckpoint {
    checkpoint: DesktopCallbackCheckpoint,
    immediate_complete: bool,
    automatic_level_complete: bool,
    automatic_act_complete: bool,
    callback_count: u64,
    active_milliseconds: u64,
    #[serde(default)]
    control_verified: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    attempted_operations: Vec<String>,
    #[serde(default)]
    attempted_requests: Vec<RequestAttempt>,
    observations: Vec<OperationObservation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RequestAttempt {
    operation: String,
    request_intent_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EvidenceEnvelope {
    format: String,
    payload: EvidencePayload,
    integrity_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EvidencePayload {
    status: String,
    observed_on: String,
    source_identity: SourceIdentity,
    implementation_identity: String,
    response_fingerprint_normalization: String,
    target_profile: CompatibilityProfile,
    import_path: DesktopImportMetadata,
    realm: String,
    verified_https_endpoint: String,
    credential_mode: String,
    disposable_character_sha256: String,
    max_active_seconds: u64,
    classification_poll_seconds: u64,
    handoff: HandoffEvidence,
    observations: Vec<OperationObservation>,
    classification: String,
    cleanup_complete: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SourceIdentity {
    tag: String,
    commit: String,
    main_pas_sha256: String,
    config_dfm_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct HandoffEvidence {
    official_client_created_identity: bool,
    official_client_stopped_before_native_reporter: bool,
    official_client_control_verified: bool,
    simultaneous_reporters_allowed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OperationObservation {
    operation: String,
    #[serde(default)]
    request_intent_sha256: String,
    request_attempts: u8,
    delivery: String,
    classification: String,
    #[serde(default)]
    outcome: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    http_status: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_category: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    public_profile_verified: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_fingerprint_sha256: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Classification {
    Normal,
    Cheater,
    Pending,
}

struct DeliveredResponse {
    status: u16,
    body: Vec<u8>,
}

struct LiveContext {
    save: DesktopValidatedSave,
    name: String,
    authentication: DesktopAccountAuthentication,
    authorization: String,
    agent: ureq::Agent,
    observations: Vec<OperationObservation>,
    poll_bound: Duration,
    official_control_verified: bool,
}

pub fn run(options: &DesktopLiveExperimentOptions) -> Result<(), DesktopLiveExperimentError> {
    if options.realm != REALM || options.stage != DesktopLiveStage::Legacy {
        return scoped::run(options);
    }
    validate_options(options)?;
    let (save_path, backup_path) = select_save_files(&options.experiment_dir)?;
    let save = match import_supported_file(&save_path)
        .map_err(|_| DesktopLiveExperimentError::UnsupportedSave)?
    {
        ImportedSave::Desktop(save) => save,
        ImportedSave::Browser(_) => return Err(DesktopLiveExperimentError::UnsupportedSave),
    };
    validate_contract(&save)?;
    validate_fresh(&save)?;
    if options.validate_only {
        println!(
            "disposable desktop save and approved Spoltog scope validated without network activity"
        );
        return Ok(());
    }

    let name = trait_value(&save, "Name")?.to_owned();
    let authentication =
        DesktopAccountAuthentication::new(&save.private.account, &save.private.password);
    let authorization = format!(
        "Basic {}",
        STANDARD.encode(format!(
            "{}:{}",
            save.private.account, save.private.password
        ))
    );
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .https_only(true)
        .max_redirects(0)
        .http_status_as_error(false)
        .timeout_global(Some(Duration::from_secs(30)))
        .build()
        .into();
    let mut context = LiveContext {
        save,
        name,
        authentication,
        authorization,
        agent,
        observations: Vec::new(),
        poll_bound: Duration::from_secs(options.classification_poll_seconds),
        official_control_verified: false,
    };

    let checkpoint_path = options.experiment_dir.join("desktop-live-checkpoint.json");
    let mut checkpoint = load_or_create_checkpoint(&checkpoint_path, &context.save)?;
    context.observations = checkpoint.observations.clone();
    if !checkpoint.attempted_operations.is_empty()
        || checkpoint
            .attempted_requests
            .iter()
            .any(|attempted| !observation_exists_for_attempt(&checkpoint.observations, attempted))
    {
        write_evidence(options, &context, "inconclusive", false)?;
        return Err(DesktopLiveExperimentError::Inconclusive);
    }
    context.official_control_verified = checkpoint.control_verified;
    if !checkpoint.control_verified {
        let verified = poll_profile(
            &context,
            PublicProfileExpectation::Contains(&options.control_motto),
        )?;
        let classification = poll_classification(&context)?;
        if !verified || classification != Classification::Normal {
            write_evidence(options, &context, "inconclusive", false)?;
            return Err(DesktopLiveExperimentError::Inconclusive);
        }
        checkpoint.control_verified = true;
        context.official_control_verified = true;
        persist_checkpoint(&checkpoint_path, &checkpoint)?;
    }
    let stop = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(SIGINT, Arc::clone(&stop))
        .map_err(|_| DesktopLiveExperimentError::PrivateState)?;
    signal_hook::flag::register(SIGTERM, Arc::clone(&stop))
        .map_err(|_| DesktopLiveExperimentError::PrivateState)?;

    write_evidence(options, &context, "running", false)?;
    if !checkpoint.immediate_complete {
        let result =
            run_immediate_operations(&mut context, &mut checkpoint, &checkpoint_path, options);
        if let Err(error) = result {
            context.observations = checkpoint.observations.clone();
            write_evidence(options, &context, "inconclusive", false)?;
            return Err(error);
        }
        checkpoint.immediate_complete = true;
        checkpoint.observations = context.observations.clone();
        persist_checkpoint(&checkpoint_path, &checkpoint)?;
        write_evidence(options, &context, "running", false)?;
    }
    if options.immediate_only {
        write_evidence(options, &context, "immediate-passing", false)?;
        println!(
            "desktop immediate live experiment completed; private checkpoint retained for an approved continuation"
        );
        return Ok(());
    }

    let mut baseline = Instant::now();
    while checkpoint.active_milliseconds < options.max_active_seconds.saturating_mul(1_000)
        && !(checkpoint.automatic_level_complete && checkpoint.automatic_act_complete)
    {
        if stop.load(Ordering::Relaxed) {
            persist_checkpoint(&checkpoint_path, &checkpoint)?;
            write_evidence(options, &context, "interrupted", false)?;
            return Err(DesktopLiveExperimentError::Interrupted);
        }
        thread::sleep(CALLBACK_INTERVAL);
        let now = Instant::now();
        let elapsed = now.saturating_duration_since(baseline);
        let elapsed_ms = i64::try_from(elapsed.as_millis()).unwrap_or(i64::MAX);
        let mut hooks = SourceDerivedDesktopHooks::traced();
        let observation = checkpoint
            .checkpoint
            .apply_progression_callback(elapsed_ms, &mut hooks)
            .map_err(|_| DesktopLiveExperimentError::Progression)?;
        checkpoint.callback_count = checkpoint.callback_count.saturating_add(1);
        checkpoint.active_milliseconds = checkpoint
            .active_milliseconds
            .saturating_add(observation.credited_milliseconds);
        for snapshot in hooks.reports() {
            let operation = match snapshot.trigger {
                DesktopReportTrigger::Level => "automatic-level",
                DesktopReportTrigger::Act => "automatic-act",
            };
            if snapshot.trigger == DesktopReportTrigger::Act
                && operation_complete(&checkpoint, operation)
            {
                continue;
            }
            if snapshot.trigger == DesktopReportTrigger::Act
                && request_attempt_count(&checkpoint, operation) > 0
            {
                write_evidence(options, &context, "inconclusive", false)?;
                return Err(DesktopLiveExperimentError::Inconclusive);
            }
            let request = report_for_snapshot(
                snapshot,
                REALM,
                &checkpoint.checkpoint.state.profile.motto,
                context.save.private.passkey,
                &context.authentication,
            )
            .map_err(|_| DesktopLiveExperimentError::Construction)?;
            let query = request.encoded_query();
            let request_intent_sha256 =
                begin_request(&mut checkpoint, &checkpoint_path, operation, &query)?;
            let result =
                deliver_report(&mut context, operation, &request_intent_sha256, query, None);
            if result.is_ok() {
                match snapshot.trigger {
                    DesktopReportTrigger::Level => checkpoint.automatic_level_complete = true,
                    DesktopReportTrigger::Act => checkpoint.automatic_act_complete = true,
                }
            }
            checkpoint.observations = context.observations.clone();
            persist_checkpoint(&checkpoint_path, &checkpoint)?;
            if let Err(error) = result {
                write_evidence(options, &context, "inconclusive", false)?;
                return Err(error);
            }
        }
        checkpoint.observations = context.observations.clone();
        persist_checkpoint(&checkpoint_path, &checkpoint)?;
        write_evidence(options, &context, "running", false)?;
        baseline = Instant::now();
    }

    if !(checkpoint.automatic_level_complete && checkpoint.automatic_act_complete) {
        write_evidence(options, &context, "inconclusive", false)?;
        return Err(DesktopLiveExperimentError::BoundExpired);
    }
    let classification = poll_classification(&context)?;
    ensure_normal(classification)?;
    fs::remove_file(&checkpoint_path).map_err(|_| DesktopLiveExperimentError::Cleanup)?;
    fs::remove_file(&save_path).map_err(|_| DesktopLiveExperimentError::Cleanup)?;
    if let Some(backup_path) = backup_path {
        fs::remove_file(backup_path).map_err(|_| DesktopLiveExperimentError::Cleanup)?;
    }
    write_evidence(options, &context, "passing", true)?;
    println!("desktop live experiment completed with sanitized passing evidence");
    Ok(())
}

fn validate_options(
    options: &DesktopLiveExperimentOptions,
) -> Result<(), DesktopLiveExperimentError> {
    if !options.confirm_disposable
        || !options.confirm_live_submission
        || !options.confirm_client_stopped
        || !options.confirm_cleanup
    {
        return Err(DesktopLiveExperimentError::MissingConfirmation);
    }
    if options.max_active_seconds < MIN_ACTIVE_SECONDS
        || options.max_active_seconds > MAX_ACTIVE_SECONDS
        || !(5..=300).contains(&options.classification_poll_seconds)
        || options.control_motto.is_empty()
        || options.control_motto.len() > MAX_MOTTO_BYTES
        || !options.control_motto.is_ascii()
        || options.motto.is_empty()
        || options.motto.len() > MAX_MOTTO_BYTES
        || !options.motto.is_ascii()
        || options.motto == options.control_motto
        || options.guild_designation.is_empty()
        || options.guild_designation.len() > MAX_GUILD_BYTES
        || !options.guild_designation.is_ascii()
        || !options.operations.is_empty()
        || options.allow_existing_disposable
        || options.allow_quest_placeholder
        || options.initial_guild_leave
        || options.preparation_reconciliation_seconds.is_some()
        || options.confirm_preparation_reconciliation
        || options.operator_join_reconciliation_seconds.is_some()
        || options.confirm_operator_accepted_join
        || options.confirm_no_competing_join
        || options.max_mutation_attempts.is_some()
        || options.guild_change_designation.is_some()
        || options.realm != REALM
        || options.stage != DesktopLiveStage::Legacy
    {
        return Err(DesktopLiveExperimentError::InvalidScope);
    }
    Ok(())
}

fn select_save_files(
    directory: &Path,
) -> Result<(PathBuf, Option<PathBuf>), DesktopLiveExperimentError> {
    let entries = fs::read_dir(directory)
        .map_err(|_| DesktopLiveExperimentError::InvalidExperimentDirectory)?;
    let mut saves = Vec::new();
    let mut backups = Vec::new();
    for entry in entries {
        let path = entry
            .map_err(|_| DesktopLiveExperimentError::InvalidExperimentDirectory)?
            .path();
        if !path.is_file() {
            continue;
        }
        match path.extension().and_then(|value| value.to_str()) {
            Some(extension) if extension.eq_ignore_ascii_case("pq") => saves.push(path),
            Some(extension) if extension.eq_ignore_ascii_case("bak") => backups.push(path),
            Some("json") => {}
            _ => return Err(DesktopLiveExperimentError::InvalidExperimentDirectory),
        }
    }
    if saves.len() != 1 || backups.len() > 1 {
        return Err(DesktopLiveExperimentError::InvalidExperimentDirectory);
    }
    Ok((saves.remove(0), backups.pop()))
}

fn validate_contract(save: &DesktopValidatedSave) -> Result<(), DesktopLiveExperimentError> {
    if save.private.realm != REALM
        || save.private.endpoint != SAVED_ENDPOINT
        || save.private.passkey <= 0
        || save.private.account.is_empty()
        || save.private.password.is_empty()
        || !save.private.account.is_ascii()
        || !save.private.password.is_ascii()
    {
        return Err(DesktopLiveExperimentError::ContractMismatch);
    }
    Ok(())
}

fn validate_fresh(save: &DesktopValidatedSave) -> Result<(), DesktopLiveExperimentError> {
    let level = trait_value(save, "Level")?
        .parse::<u64>()
        .map_err(|_| DesktopLiveExperimentError::NotFresh)?;
    if !is_new_character_handoff(level, save.plots.len(), save.quests.len()) {
        return Err(DesktopLiveExperimentError::NotFresh);
    }
    Ok(())
}

fn is_new_character_handoff(level: u64, plot_count: usize, quest_count: usize) -> bool {
    level == 1 && (1..=MAX_HANDOFF_PLOTS).contains(&plot_count) && quest_count <= 1
}

fn trait_value<'a>(
    save: &'a DesktopValidatedSave,
    caption: &str,
) -> Result<&'a str, DesktopLiveExperimentError> {
    save.traits
        .iter()
        .find(|row| row.caption == caption)
        .and_then(|row| row.subitems.first())
        .map(String::as_str)
        .ok_or(DesktopLiveExperimentError::NotFresh)
}

fn load_or_create_checkpoint(
    path: &Path,
    save: &DesktopValidatedSave,
) -> Result<PrivateCheckpoint, DesktopLiveExperimentError> {
    if path.exists() {
        let content =
            fs::read_to_string(path).map_err(|_| DesktopLiveExperimentError::PrivateState)?;
        let mut checkpoint: PrivateCheckpoint =
            serde_json::from_str(&content).map_err(|_| DesktopLiveExperimentError::PrivateState)?;
        if !checkpoint.control_verified
            && !checkpoint.immediate_complete
            && !checkpoint.automatic_level_complete
            && !checkpoint.automatic_act_complete
            && checkpoint.callback_count == 0
            && checkpoint.active_milliseconds == 0
            && checkpoint.attempted_operations.is_empty()
            && checkpoint.attempted_requests.is_empty()
            && checkpoint.observations.is_empty()
        {
            checkpoint.checkpoint.state = DesktopCanonicalState::from(save);
            persist_checkpoint(path, &checkpoint)?;
        }
        return Ok(checkpoint);
    }
    let mut random = OsRandom::open().map_err(|_| DesktopLiveExperimentError::PrivateState)?;
    let checkpoint = PrivateCheckpoint {
        checkpoint: DesktopCallbackCheckpoint {
            state: DesktopCanonicalState::from(save),
            random: DesktopRandomState(
                random
                    .next_u32()
                    .map_err(|_| DesktopLiveExperimentError::PrivateState)?,
            ),
        },
        immediate_complete: false,
        automatic_level_complete: false,
        automatic_act_complete: false,
        callback_count: 0,
        active_milliseconds: 0,
        control_verified: false,
        attempted_operations: Vec::new(),
        attempted_requests: Vec::new(),
        observations: Vec::new(),
    };
    persist_checkpoint(path, &checkpoint)?;
    Ok(checkpoint)
}

fn persist_checkpoint(
    path: &Path,
    checkpoint: &PrivateCheckpoint,
) -> Result<(), DesktopLiveExperimentError> {
    let temporary = path.with_extension("json.tmp");
    let content =
        serde_json::to_vec(checkpoint).map_err(|_| DesktopLiveExperimentError::PrivateState)?;
    let mut file =
        fs::File::create(&temporary).map_err(|_| DesktopLiveExperimentError::PrivateState)?;
    file.write_all(&content)
        .and_then(|_| file.sync_all())
        .map_err(|_| DesktopLiveExperimentError::PrivateState)?;
    fs::rename(temporary, path).map_err(|_| DesktopLiveExperimentError::PrivateState)
}

fn run_immediate_operations(
    context: &mut LiveContext,
    checkpoint: &mut PrivateCheckpoint,
    checkpoint_path: &Path,
    options: &DesktopLiveExperimentOptions,
) -> Result<(), DesktopLiveExperimentError> {
    ensure_normal(poll_classification(context)?)?;
    let state = checkpoint.checkpoint.state.clone();

    if !operation_complete(checkpoint, "manual-brag") {
        let manual = report(
            &state,
            DesktopReportOperation::Manual,
            REALM,
            &state.profile.motto,
            context.save.private.passkey,
            &context.authentication,
        )
        .map_err(|_| DesktopLiveExperimentError::Construction)?;
        let query = manual.encoded_query();
        let request_intent_sha256 =
            begin_request(checkpoint, checkpoint_path, "manual-brag", &query)?;
        let result = deliver_report(context, "manual-brag", &request_intent_sha256, query, None);
        finish_operation(context, checkpoint, checkpoint_path)?;
        result?;
    }

    let selected_motto = options.motto.as_str();
    if !operation_complete(checkpoint, "motto-set") {
        if request_attempt_count(checkpoint, "motto-set") >= MAX_DISTINCT_MOTTO_ATTEMPTS {
            return Err(DesktopLiveExperimentError::Inconclusive);
        }
        let motto = report(
            &state,
            DesktopReportOperation::Motto,
            REALM,
            selected_motto,
            context.save.private.passkey,
            &context.authentication,
        )
        .map_err(|_| DesktopLiveExperimentError::Construction)?;
        let query = motto.encoded_query();
        let request_intent_sha256 =
            begin_request(checkpoint, checkpoint_path, "motto-set", &query)?;
        let result = deliver_report(
            context,
            "motto-set",
            &request_intent_sha256,
            query,
            Some(PublicProfileExpectation::Contains(selected_motto)),
        );
        if result.is_ok() {
            checkpoint.checkpoint.state.profile.motto = selected_motto.to_owned();
        }
        finish_operation(context, checkpoint, checkpoint_path)?;
        result?;
    }

    if !operation_complete(checkpoint, "motto-clear") {
        let clear_motto = report(
            &state,
            DesktopReportOperation::Motto,
            REALM,
            "",
            context.save.private.passkey,
            &context.authentication,
        )
        .map_err(|_| DesktopLiveExperimentError::Construction)?;
        let query = clear_motto.encoded_query();
        let request_intent_sha256 =
            begin_request(checkpoint, checkpoint_path, "motto-clear", &query)?;
        let result = deliver_report(
            context,
            "motto-clear",
            &request_intent_sha256,
            query,
            Some(PublicProfileExpectation::Absent(selected_motto)),
        );
        if result.is_ok() {
            checkpoint.checkpoint.state.profile.motto.clear();
        }
        finish_operation(context, checkpoint, checkpoint_path)?;
        result?;
    }

    for (operation, prior, designation, expectation) in [
        (
            "guild-accepted-nonempty",
            state.profile.guild.as_str(),
            options.guild_designation.as_str(),
            PublicProfileExpectation::Contains(options.guild_designation.as_str()),
        ),
        (
            "guild-rejected-invalid",
            options.guild_designation.as_str(),
            "__gyrognome-invalid__",
            PublicProfileExpectation::Contains(options.guild_designation.as_str()),
        ),
        (
            "guild-accepted-empty",
            options.guild_designation.as_str(),
            "",
            PublicProfileExpectation::Absent(options.guild_designation.as_str()),
        ),
    ] {
        if operation_complete(checkpoint, operation) {
            continue;
        }
        let request = guild_request(
            &state,
            REALM,
            designation,
            context.save.private.passkey,
            &context.authentication,
        )
        .map_err(|_| DesktopLiveExperimentError::Construction)?;
        let query = request.encoded_query();
        let request_intent_sha256 = begin_request(checkpoint, checkpoint_path, operation, &query)?;
        let result = deliver_guild(
            context,
            operation,
            &request_intent_sha256,
            prior,
            designation,
            expectation,
            query,
        );
        finish_operation(context, checkpoint, checkpoint_path)?;
        result?;
    }
    Ok(())
}

fn operation_complete(checkpoint: &PrivateCheckpoint, operation: &str) -> bool {
    checkpoint.observations.iter().any(|observation| {
        observation.operation == operation
            && matches!(
                observation.outcome.as_str(),
                "verified" | "delivered-normal"
            )
    })
}

fn request_attempt_count(checkpoint: &PrivateCheckpoint, operation: &str) -> u8 {
    request_attempt_count_for(&checkpoint.attempted_requests, operation)
}

fn request_attempt_count_for(attempts: &[RequestAttempt], operation: &str) -> u8 {
    attempts
        .iter()
        .filter(|attempt| attempt.operation == operation)
        .count()
        .try_into()
        .unwrap_or(u8::MAX)
}

fn begin_request(
    checkpoint: &mut PrivateCheckpoint,
    checkpoint_path: &Path,
    operation: &str,
    query: &str,
) -> Result<String, DesktopLiveExperimentError> {
    let request_intent_sha256 = request_intent_sha256(operation, query);
    if request_was_attempted(
        &checkpoint.attempted_requests,
        operation,
        &request_intent_sha256,
    ) {
        return Err(DesktopLiveExperimentError::Inconclusive);
    }
    checkpoint.attempted_requests.push(RequestAttempt {
        operation: operation.to_owned(),
        request_intent_sha256: request_intent_sha256.clone(),
    });
    persist_checkpoint(checkpoint_path, checkpoint)?;
    Ok(request_intent_sha256)
}

fn request_was_attempted(
    attempts: &[RequestAttempt],
    operation: &str,
    request_intent_sha256: &str,
) -> bool {
    attempts.iter().any(|attempted| {
        attempted.operation == operation && attempted.request_intent_sha256 == request_intent_sha256
    })
}

fn observation_exists_for_attempt(
    observations: &[OperationObservation],
    attempt: &RequestAttempt,
) -> bool {
    observations.iter().any(|observation| {
        observation.operation == attempt.operation
            && observation.request_intent_sha256 == attempt.request_intent_sha256
    })
}

fn finish_operation(
    context: &LiveContext,
    checkpoint: &mut PrivateCheckpoint,
    checkpoint_path: &Path,
) -> Result<(), DesktopLiveExperimentError> {
    checkpoint.observations = context.observations.clone();
    persist_checkpoint(checkpoint_path, checkpoint)
}

enum PublicProfileExpectation<'a> {
    Contains(&'a str),
    Absent(&'a str),
}

fn deliver_guild(
    context: &mut LiveContext,
    operation: &str,
    request_intent_sha256: &str,
    prior_designation: &str,
    designation: &str,
    expectation: PublicProfileExpectation<'_>,
    query: String,
) -> Result<(), DesktopLiveExperimentError> {
    let response = deliver(context, &query)?;
    let fingerprint =
        guild_response_fingerprint(context, &response.body, prior_designation, designation);
    let response_category = response_category(response.status, &response.body);
    if !(200..=299).contains(&response.status) {
        context.observations.push(OperationObservation {
            operation: operation.to_owned(),
            request_intent_sha256: request_intent_sha256.to_owned(),
            request_attempts: 1,
            delivery: "endpoint-rejected".to_owned(),
            classification: "pending".to_owned(),
            outcome: "rejected".to_owned(),
            http_status: Some(response.status),
            response_category: Some(response_category),
            public_profile_verified: None,
            response_fingerprint_sha256: Some(fingerprint),
        });
        return Err(DesktopLiveExperimentError::Delivery);
    }
    let observation_index = context.observations.len();
    context.observations.push(OperationObservation {
        operation: operation.to_owned(),
        request_intent_sha256: request_intent_sha256.to_owned(),
        request_attempts: 1,
        delivery: "delivered".to_owned(),
        classification: "pending".to_owned(),
        outcome: "delivered-unverified".to_owned(),
        http_status: Some(response.status),
        response_category: Some(response_category),
        public_profile_verified: None,
        response_fingerprint_sha256: Some(fingerprint),
    });
    let verified = poll_profile(context, expectation)?;
    let classification = poll_classification(context)?;
    let observation = &mut context.observations[observation_index];
    observation.classification = classification_label(classification).to_owned();
    observation.outcome = if verified && classification == Classification::Normal {
        "verified".to_owned()
    } else {
        "inconclusive".to_owned()
    };
    observation.public_profile_verified = Some(verified);
    if !verified {
        return Err(DesktopLiveExperimentError::Inconclusive);
    }
    ensure_normal(classification)
}

fn deliver_report(
    context: &mut LiveContext,
    operation: &str,
    request_intent_sha256: &str,
    query: String,
    profile_expectation: Option<PublicProfileExpectation<'_>>,
) -> Result<(), DesktopLiveExperimentError> {
    let submitted_value = profile_expectation
        .as_ref()
        .map(|expectation| match expectation {
            PublicProfileExpectation::Contains(value) | PublicProfileExpectation::Absent(value) => {
                *value
            }
        });
    let response = deliver(context, &query)?;
    let submitted_values = submitted_value.into_iter().collect::<Vec<_>>();
    let fingerprint = response_fingerprint(context, &response.body, &submitted_values);
    let response_category = response_category(response.status, &response.body);
    if !(200..=299).contains(&response.status) {
        context.observations.push(OperationObservation {
            operation: operation.to_owned(),
            request_intent_sha256: request_intent_sha256.to_owned(),
            request_attempts: 1,
            delivery: "endpoint-rejected".to_owned(),
            classification: "pending".to_owned(),
            outcome: "rejected".to_owned(),
            http_status: Some(response.status),
            response_category: Some(response_category),
            public_profile_verified: None,
            response_fingerprint_sha256: Some(fingerprint),
        });
        return Err(DesktopLiveExperimentError::Delivery);
    }
    let observation_index = context.observations.len();
    context.observations.push(OperationObservation {
        operation: operation.to_owned(),
        request_intent_sha256: request_intent_sha256.to_owned(),
        request_attempts: 1,
        delivery: "delivered".to_owned(),
        classification: "pending".to_owned(),
        outcome: "delivered-unverified".to_owned(),
        http_status: Some(response.status),
        response_category: Some(response_category),
        public_profile_verified: None,
        response_fingerprint_sha256: Some(fingerprint),
    });
    let profile_verified = match profile_expectation {
        Some(expectation) => Some(poll_profile(context, expectation)?),
        None => None,
    };
    let classification = poll_classification(context)?;
    let verified = profile_verified.unwrap_or(true) && classification == Classification::Normal;
    let observation = &mut context.observations[observation_index];
    observation.classification = classification_label(classification).to_owned();
    observation.outcome = if verified {
        if profile_verified.is_some() {
            "verified".to_owned()
        } else {
            "delivered-normal".to_owned()
        }
    } else {
        "inconclusive".to_owned()
    };
    observation.public_profile_verified = profile_verified;
    ensure_normal(classification)?;
    if profile_verified == Some(false) {
        return Err(DesktopLiveExperimentError::Inconclusive);
    }
    Ok(())
}

fn deliver(
    context: &LiveContext,
    query: &str,
) -> Result<DeliveredResponse, DesktopLiveExperimentError> {
    let mut endpoint =
        Url::parse(VERIFIED_ENDPOINT).map_err(|_| DesktopLiveExperimentError::ContractMismatch)?;
    endpoint.set_query(Some(query));
    let mut response = context
        .agent
        .get(endpoint.as_str())
        .header("Authorization", &context.authorization)
        .call()
        .map_err(|_| DesktopLiveExperimentError::Delivery)?;
    let status = response.status().as_u16();
    let mut body = Vec::new();
    response
        .body_mut()
        .as_reader()
        .take(MAX_RESPONSE_BYTES + 1)
        .read_to_end(&mut body)
        .map_err(|_| DesktopLiveExperimentError::Delivery)?;
    if body.len() as u64 > MAX_RESPONSE_BYTES {
        return Err(DesktopLiveExperimentError::Delivery);
    }
    Ok(DeliveredResponse { status, body })
}

fn poll_classification(
    context: &LiveContext,
) -> Result<Classification, DesktopLiveExperimentError> {
    let deadline = Instant::now() + context.poll_bound;
    loop {
        match public_character_page(context) {
            Ok(page) => {
                let classification = classify_page(&page, &context.name);
                if classification != Classification::Pending || Instant::now() >= deadline {
                    return Ok(classification);
                }
            }
            Err(DesktopLiveExperimentError::Inconclusive) if Instant::now() < deadline => {}
            Err(error) => return Err(error),
        }
        if Instant::now() >= deadline {
            return Ok(Classification::Pending);
        }
        thread::sleep(PUBLIC_POLL_INTERVAL);
    }
}

fn poll_profile(
    context: &LiveContext,
    expectation: PublicProfileExpectation<'_>,
) -> Result<bool, DesktopLiveExperimentError> {
    let deadline = Instant::now() + context.poll_bound;
    loop {
        match public_character_page(context) {
            Ok(page) => {
                let row = character_row(&page, &context.name);
                let matches = row.is_some_and(|row| match expectation {
                    PublicProfileExpectation::Contains(value) => row.contains(value),
                    PublicProfileExpectation::Absent(value) => !row.contains(value),
                });
                if matches {
                    return Ok(true);
                }
            }
            Err(DesktopLiveExperimentError::Inconclusive) if Instant::now() < deadline => {}
            Err(error) => return Err(error),
        }
        if Instant::now() >= deadline {
            return Ok(false);
        }
        thread::sleep(PUBLIC_POLL_INTERVAL);
    }
}

fn public_character_page(context: &LiveContext) -> Result<String, DesktopLiveExperimentError> {
    let mut endpoint =
        Url::parse(VERIFIED_ENDPOINT).map_err(|_| DesktopLiveExperimentError::ContractMismatch)?;
    endpoint
        .query_pairs_mut()
        .append_pair("name", &context.name);
    let mut response = context
        .agent
        .get(endpoint.as_str())
        .call()
        .map_err(|_| DesktopLiveExperimentError::Inconclusive)?;
    if !response.status().is_success() {
        return Err(DesktopLiveExperimentError::Inconclusive);
    }
    let mut body = String::new();
    response
        .body_mut()
        .as_reader()
        .take(256 * 1024)
        .read_to_string(&mut body)
        .map_err(|_| DesktopLiveExperimentError::Inconclusive)?;
    Ok(body)
}

fn classify_page(page: &str, name: &str) -> Classification {
    let Some(name_offset) = page.find(&html_escape(name)) else {
        return Classification::Pending;
    };
    let prefix = &page[..name_offset];
    let fame = prefix.rfind("Hall of Fame");
    let infamy = prefix.rfind("Hall of Infamy");
    match (fame, infamy) {
        (Some(fame), Some(infamy)) if infamy > fame => Classification::Cheater,
        (Some(_), _) => Classification::Normal,
        (_, Some(_)) => Classification::Cheater,
        _ => Classification::Pending,
    }
}

fn character_row<'a>(page: &'a str, name: &str) -> Option<&'a str> {
    let name_offset = page.find(&html_escape(name))?;
    let start = page[..name_offset].rfind("<tr")?;
    let remainder = &page[name_offset..];
    let next_row = remainder.find("<tr").map(|offset| name_offset + offset);
    let table_end = remainder
        .find("</table>")
        .map(|offset| name_offset + offset);
    let end = match (next_row, table_end) {
        (Some(next_row), Some(table_end)) => next_row.min(table_end),
        (Some(next_row), None) => next_row,
        (None, Some(table_end)) => table_end,
        (None, None) => page.len(),
    };
    page.get(start..end)
}

fn ensure_normal(classification: Classification) -> Result<(), DesktopLiveExperimentError> {
    match classification {
        Classification::Normal => Ok(()),
        Classification::Cheater => Err(DesktopLiveExperimentError::Cheater),
        Classification::Pending => Err(DesktopLiveExperimentError::Inconclusive),
    }
}

fn classification_label(classification: Classification) -> &'static str {
    match classification {
        Classification::Normal => "normal",
        Classification::Cheater => "cheater",
        Classification::Pending => "pending",
    }
}

fn request_intent_sha256(operation: &str, query: &str) -> String {
    sha256(format!("{operation}\0{query}").as_bytes())
}

fn response_category(status: u16, body: &[u8]) -> String {
    match status {
        401 | 403 => "http-authentication-rejected",
        200..=299 if body.is_empty() => "http-success-empty",
        200..=299 if std::str::from_utf8(body).is_ok() => "http-success-text",
        200..=299 => "http-success-nontext",
        _ => "http-endpoint-rejected",
    }
    .to_owned()
}

fn response_fingerprint(context: &LiveContext, body: &[u8], submitted_values: &[&str]) -> String {
    let passkey = context.save.private.passkey.to_string();
    let sensitive_values = [
        context.name.as_str(),
        context.save.private.account.as_str(),
        context.save.private.password.as_str(),
        context.authorization.as_str(),
        passkey.as_str(),
    ];
    normalized_response_fingerprint(
        body,
        sensitive_values
            .into_iter()
            .chain(submitted_values.iter().copied()),
    )
}

fn guild_response_fingerprint(
    context: &LiveContext,
    body: &[u8],
    prior_guild: &str,
    submitted_guild: &str,
) -> String {
    let passkey = context.save.private.passkey.to_string();
    DesktopGuildFingerprintValues {
        character_name: &context.name,
        account: &context.save.private.account,
        password: &context.save.private.password,
        authorization: &context.authorization,
        passkey: &passkey,
        prior_guild,
        submitted_guild,
    }
    .fingerprint(body)
}

fn write_evidence(
    options: &DesktopLiveExperimentOptions,
    context: &LiveContext,
    status: &str,
    cleanup_complete: bool,
) -> Result<(), DesktopLiveExperimentError> {
    let payload = EvidencePayload {
        status: status.to_owned(),
        observed_on: current_utc_date(),
        source_identity: SourceIdentity {
            tag: SOURCE_TAG.to_owned(),
            commit: SOURCE_COMMIT.to_owned(),
            main_pas_sha256: MAIN_PAS_SHA256.to_owned(),
            config_dfm_sha256: CONFIG_DFM_SHA256.to_owned(),
        },
        implementation_identity: DESKTOP_ONLINE_IMPLEMENTATION_ID.to_owned(),
        response_fingerprint_normalization: DESKTOP_RESPONSE_FINGERPRINT_VERSION.to_owned(),
        target_profile: CompatibilityProfile::Desktop644,
        import_path: DesktopImportMetadata::from_validated(&context.save.adaptations),
        realm: REALM.to_owned(),
        verified_https_endpoint: VERIFIED_ENDPOINT.to_owned(),
        credential_mode: "http-basic-from-legacy-url-userinfo".to_owned(),
        disposable_character_sha256: sha256(context.name.as_bytes()),
        max_active_seconds: options.max_active_seconds,
        classification_poll_seconds: options.classification_poll_seconds,
        handoff: HandoffEvidence {
            official_client_created_identity: true,
            official_client_stopped_before_native_reporter: true,
            official_client_control_verified: context.official_control_verified,
            simultaneous_reporters_allowed: false,
        },
        observations: context.observations.clone(),
        classification: if matches!(status, "passing" | "immediate-passing") {
            "normal".to_owned()
        } else {
            "pending".to_owned()
        },
        cleanup_complete,
    };
    let payload_bytes =
        serde_json::to_vec(&payload).map_err(|_| DesktopLiveExperimentError::PrivateState)?;
    let envelope = EvidenceEnvelope {
        format: FORMAT.to_owned(),
        integrity_sha256: sha256(&payload_bytes),
        payload,
    };
    let temporary = options.evidence_path.with_extension("json.tmp");
    let content = serde_json::to_vec_pretty(&envelope)
        .map_err(|_| DesktopLiveExperimentError::PrivateState)?;
    fs::write(&temporary, content).map_err(|_| DesktopLiveExperimentError::PrivateState)?;
    fs::rename(temporary, &options.evidence_path)
        .map_err(|_| DesktopLiveExperimentError::PrivateState)
}

fn sha256(bytes: &[u8]) -> String {
    digest(&SHA256, bytes)
        .as_ref()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn html_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn current_utc_date() -> String {
    let days = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() / 86_400)
        .unwrap_or_default();
    let (year, month, day) = civil_from_days(days as i64);
    format!("{year:04}-{month:02}-{day:02}")
}

fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let days = days + 719_468;
    let era = if days >= 0 { days } else { days - 146_096 } / 146_097;
    let day_of_era = days - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let mut year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    year += i64::from(month <= 2);
    (year, month, day)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn options_require_full_live_scope() {
        let mut options = DesktopLiveExperimentOptions {
            allow_existing_disposable: false,
            allow_quest_placeholder: false,
            initial_guild_leave: false,
            preparation_reconciliation_seconds: None,
            confirm_preparation_reconciliation: false,
            operator_join_reconciliation_seconds: None,
            confirm_operator_accepted_join: false,
            confirm_no_competing_join: false,
            realm: REALM.to_owned(),
            stage: DesktopLiveStage::Legacy,
            operations: vec![],
            max_mutation_attempts: None,
            manual_motto: "Synthetic manual".to_owned(),
            guild_change_designation: None,
            confirm_disposable: true,
            confirm_live_submission: true,
            confirm_client_stopped: true,
            confirm_cleanup: true,
            validate_only: false,
            immediate_only: false,
            experiment_dir: PathBuf::from("private"),
            evidence_path: PathBuf::from("evidence.json"),
            control_motto: "Gyrognome official control".to_owned(),
            motto: "Gyrognome disposable test".to_owned(),
            guild_designation: "Panda!".to_owned(),
            max_active_seconds: 28_800,
            classification_poll_seconds: 60,
        };
        assert!(validate_options(&options).is_ok());
        options.allow_quest_placeholder = true;
        assert!(matches!(
            validate_options(&options),
            Err(DesktopLiveExperimentError::InvalidScope)
        ));
        options.allow_quest_placeholder = false;
        options.confirm_cleanup = false;
        assert!(matches!(
            validate_options(&options),
            Err(DesktopLiveExperimentError::MissingConfirmation)
        ));
        options.confirm_cleanup = true;
        options.max_active_seconds = 300;
        assert!(matches!(
            validate_options(&options),
            Err(DesktopLiveExperimentError::InvalidScope)
        ));
        options.max_active_seconds = 28_800;
        options.guild_designation.clear();
        assert!(matches!(
            validate_options(&options),
            Err(DesktopLiveExperimentError::InvalidScope)
        ));
    }

    #[test]
    fn classification_uses_the_heading_before_the_matching_row() {
        let normal = "<h1>Hall of Fame</h1><table><tr><td>Disposable</td></tr></table><h1>Hall of Infamy</h1>";
        let cheater = "<h1>Hall of Fame</h1><h1>Hall of Infamy</h1><table><tr><td>Disposable</td></tr></table>";
        assert_eq!(classify_page(normal, "Disposable"), Classification::Normal);
        assert_eq!(
            classify_page(cheater, "Disposable"),
            Classification::Cheater
        );
        assert_eq!(classify_page(normal, "Missing"), Classification::Pending);
    }

    #[test]
    fn character_row_supports_the_official_omitted_row_end_markup() {
        let page = "<table><tr><td>Previous<td>Old motto<tr class=selected><td>Disposable<td>Current motto<tr><td>Next</table>";
        let row = character_row(page, "Disposable").unwrap();

        assert!(row.contains("Current motto"));
        assert!(!row.contains("Old motto"));
        assert!(!row.contains("Next"));
    }

    #[test]
    fn fresh_handoff_allows_only_bounded_official_client_progress() {
        assert!(is_new_character_handoff(1, 1, 0));
        assert!(is_new_character_handoff(1, 2, 1));
        assert!(!is_new_character_handoff(1, 0, 0));
        assert!(!is_new_character_handoff(1, 3, 1));
        assert!(!is_new_character_handoff(2, 2, 1));
        assert!(!is_new_character_handoff(1, 2, 2));
    }

    #[test]
    fn guild_fingerprint_removes_designations() {
        let first =
            normalized_response_fingerprint(b"Accepted Old Guild Panda!", ["Old Guild", "Panda!"]);
        let second = normalized_response_fingerprint(
            b"Accepted Other Guild Other!",
            ["Other Guild", "Other!"],
        );
        assert_eq!(first, second);
    }

    #[test]
    fn request_intents_block_exact_replay_but_allow_a_distinct_motto() {
        let first = request_intent_sha256("motto-set", "t=m&m=First&p=1");
        let second = request_intent_sha256("motto-set", "t=m&m=Second&p=1");
        let attempts = vec![RequestAttempt {
            operation: "motto-set".to_owned(),
            request_intent_sha256: first.clone(),
        }];

        assert!(request_was_attempted(&attempts, "motto-set", &first));
        assert!(!request_was_attempted(&attempts, "motto-set", &second));
        assert_ne!(first, second);
        assert_ne!(
            first,
            request_intent_sha256("motto-clear", "t=m&m=First&p=1")
        );
        assert_eq!(request_attempt_count_for(&attempts, "motto-set"), 1);
    }

    #[test]
    fn automatic_levels_may_repeat_but_the_act_request_is_single() {
        let attempts = vec![
            RequestAttempt {
                operation: "automatic-level".to_owned(),
                request_intent_sha256: "first-level".to_owned(),
            },
            RequestAttempt {
                operation: "automatic-level".to_owned(),
                request_intent_sha256: "second-level".to_owned(),
            },
            RequestAttempt {
                operation: "automatic-act".to_owned(),
                request_intent_sha256: "first-act".to_owned(),
            },
        ];

        assert_eq!(request_attempt_count_for(&attempts, "automatic-level"), 2);
        assert_eq!(request_attempt_count_for(&attempts, "automatic-act"), 1);
        assert!(!request_was_attempted(
            &attempts,
            "automatic-level",
            "third-level"
        ));
        assert!(request_was_attempted(
            &attempts,
            "automatic-act",
            "first-act"
        ));
    }

    #[test]
    fn response_metadata_is_sanitized_and_coarse() {
        let fingerprint = normalized_response_fingerprint(
            b"Accepted SecretMotto for SecretCharacter",
            ["SecretMotto", "SecretCharacter"],
        );
        let observation = OperationObservation {
            operation: "motto-set".to_owned(),
            request_intent_sha256: request_intent_sha256("motto-set", "t=m&m=SecretMotto&p=12345"),
            request_attempts: 1,
            delivery: "delivered".to_owned(),
            classification: "normal".to_owned(),
            outcome: "verified".to_owned(),
            http_status: Some(200),
            response_category: Some(response_category(200, b"Accepted SecretMotto")),
            public_profile_verified: Some(true),
            response_fingerprint_sha256: Some(fingerprint),
        };
        let serialized = serde_json::to_string(&observation).unwrap();

        assert!(!serialized.contains("SecretMotto"));
        assert!(!serialized.contains("SecretCharacter"));
        assert!(!serialized.contains("12345"));
        assert!(!serialized.contains("Accepted"));
        assert!(serialized.contains("http-success-text"));
        assert_eq!(
            response_category(401, b"denied"),
            "http-authentication-rejected"
        );
        assert_eq!(response_category(500, b"error"), "http-endpoint-rejected");
    }

    #[test]
    fn date_conversion_matches_unix_epoch() {
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        assert_eq!(civil_from_days(20_357), (2025, 9, 26));
    }
}
