use std::{
    collections::BTreeMap,
    fs::{self, OpenOptions},
    io::{Read, Write},
    os::unix::fs::{OpenOptionsExt, PermissionsExt},
};

use super::*;
use crate::{
    desktop_contract::{DesktopRealmContract, realm_contract},
    desktop_eligibility::{DesktopOnlineOperation, desktop_request_text_is_ascii},
    desktop_evidence::{
        DesktopEvidenceCase, DesktopGuildFingerprints, DesktopOperationEvidenceObservation,
        encode_operation_evidence,
    },
    desktop_profile::{pemptus_public_row, public_guild_for_realm, public_guild_matches},
    desktop_transport::{
        DesktopTransportCredentials, deliver_verified_desktop_request,
        resolve_verified_desktop_endpoint,
    },
    pemptus_acceptance_probe::{Classification as PublicClassification, parse_observation},
};

const CHECKPOINT: &str = "scoped-native-checkpoint.json";

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct StageCheckpoint {
    scope_sha256: String,
    callback: DesktopCallbackCheckpoint,
    active_milliseconds: u64,
    attempts: Vec<String>,
    observations: Vec<StageObservation>,
    pending_callback: bool,
    last_public_cells: Option<Vec<String>>,
    #[serde(default)]
    preparation_recovery: Option<PreparationRecovery>,
    #[serde(default)]
    operator_accepted_join: Option<OperatorAcceptedJoin>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PreparationRecovery {
    intent_sha256: String,
    authorized_at_unix_milliseconds: u64,
    observed_at_unix_milliseconds: u64,
    read_only_window_seconds: u64,
}

impl PreparationRecovery {
    fn within_bound(&self) -> bool {
        self.read_only_window_seconds == 60
            && self.observed_at_unix_milliseconds >= self.authorized_at_unix_milliseconds
            && self.observed_at_unix_milliseconds - self.authorized_at_unix_milliseconds <= 60_000
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct OperatorAcceptedJoin {
    reconciliation: PreparationRecovery,
    no_competing_join_request_confirmed: bool,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct StageObservation {
    intent_sha256: String,
    // None is preparatory guild cleanup, not operation evidence.
    case: Option<DesktopEvidenceCase>,
    fingerprint_sha256: String,
    observed_on: String,
}

struct Context<'a> {
    options: &'a DesktopLiveExperimentOptions,
    contract: DesktopRealmContract,
    save: DesktopValidatedSave,
    name: String,
    checkpoint_path: PathBuf,
    checkpoint: StageCheckpoint,
    transport: &'a dyn StageTransport,
}

trait StageTransport {
    fn page(
        &self,
        contract: DesktopRealmContract,
        name: &str,
        deadline: Instant,
    ) -> Result<String, DesktopLiveExperimentError>;
    fn deliver(
        &self,
        target: &crate::desktop_transport::VerifiedDesktopEndpoint,
        query: &str,
        credentials: &DesktopTransportCredentials,
    ) -> Result<crate::desktop_transport::DesktopHttpResponse, DesktopLiveExperimentError>;
}

struct NativeStageTransport;

impl StageTransport for NativeStageTransport {
    fn deliver(
        &self,
        target: &crate::desktop_transport::VerifiedDesktopEndpoint,
        query: &str,
        credentials: &DesktopTransportCredentials,
    ) -> Result<crate::desktop_transport::DesktopHttpResponse, DesktopLiveExperimentError> {
        deliver_verified_desktop_request(target, query, credentials)
            .map_err(|_| DesktopLiveExperimentError::Delivery)
    }

    fn page(
        &self,
        contract: DesktopRealmContract,
        name: &str,
        deadline: Instant,
    ) -> Result<String, DesktopLiveExperimentError> {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Err(DesktopLiveExperimentError::Inconclusive);
        }
        let mut endpoint = Url::parse(contract.https_endpoint)
            .map_err(|_| DesktopLiveExperimentError::ContractMismatch)?;
        endpoint.query_pairs_mut().append_pair("name", name);
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .https_only(true)
            .max_redirects(0)
            .http_status_as_error(false)
            .timeout_global(Some(remaining.min(Duration::from_secs(30))))
            .build()
            .into();
        let mut response = agent
            .get(endpoint.as_str())
            .call()
            .map_err(|_| DesktopLiveExperimentError::Inconclusive)?;
        if !response.status().is_success() {
            return Err(DesktopLiveExperimentError::Inconclusive);
        }
        let mut bytes = vec![];
        response
            .body_mut()
            .as_reader()
            .take(256 * 1024 + 1)
            .read_to_end(&mut bytes)
            .map_err(|_| DesktopLiveExperimentError::Inconclusive)?;
        if bytes.len() > 256 * 1024 || Instant::now() > deadline {
            return Err(DesktopLiveExperimentError::Inconclusive);
        }
        String::from_utf8(bytes).map_err(|_| DesktopLiveExperimentError::Inconclusive)
    }
}

pub(super) fn validate_scope(
    options: &DesktopLiveExperimentOptions,
) -> Result<DesktopRealmContract, DesktopLiveExperimentError> {
    if !options.confirm_disposable
        || !options.confirm_live_submission
        || !options.confirm_client_stopped
        || !options.confirm_cleanup
    {
        return Err(DesktopLiveExperimentError::MissingConfirmation);
    }
    let contract =
        realm_contract(&options.realm).map_err(|_| DesktopLiveExperimentError::InvalidScope)?;
    if (options.allow_existing_disposable
        && (contract.realm != "Pemptus" || options.stage != DesktopLiveStage::Immediate))
        || (options.initial_guild_leave && !options.allow_existing_disposable)
    {
        return Err(DesktopLiveExperimentError::InvalidScope);
    }
    if options.allow_quest_placeholder
        && (contract.realm != "Pemptus" || options.stage != DesktopLiveStage::Progression)
    {
        return Err(DesktopLiveExperimentError::InvalidScope);
    }
    if options.preparation_reconciliation_seconds.is_some()
        || options.confirm_preparation_reconciliation
    {
        if options.preparation_reconciliation_seconds != Some(60)
            || !options.confirm_preparation_reconciliation
            || !options.allow_existing_disposable
            || !options.initial_guild_leave
            || contract.realm != "Pemptus"
            || options.stage != DesktopLiveStage::Immediate
            || options.max_mutation_attempts != Some(8)
            || options.operations.len() != 3
            || ![
                DesktopOnlineOperation::ManualBrag,
                DesktopOnlineOperation::Motto,
                DesktopOnlineOperation::Guild,
            ]
            .iter()
            .all(|operation| options.operations.contains(operation))
        {
            return Err(DesktopLiveExperimentError::InvalidScope);
        }
    }
    if options.operator_join_reconciliation_seconds.is_some()
        || options.confirm_operator_accepted_join
        || options.confirm_no_competing_join
    {
        if options.operator_join_reconciliation_seconds != Some(60)
            || !options.confirm_operator_accepted_join
            || !options.confirm_no_competing_join
            || contract.realm != "Pemptus"
            || options.stage != DesktopLiveStage::Immediate
            || !options.allow_existing_disposable
            || !options.initial_guild_leave
            || options.max_mutation_attempts != Some(8)
            || options.classification_poll_seconds != 60
            || options.operations
                != [
                    DesktopOnlineOperation::ManualBrag,
                    DesktopOnlineOperation::Motto,
                    DesktopOnlineOperation::Guild,
                ]
            || !options.control_motto.is_empty()
            || options.guild_designation != "BEERGuild"
            || options.guild_change_designation.as_deref() != Some("QoD")
        {
            return Err(DesktopLiveExperimentError::InvalidScope);
        }
    }
    if options.immediate_only
        || options.stage == DesktopLiveStage::Legacy
        || options.operations.is_empty()
        || options
            .operations
            .iter()
            .enumerate()
            .any(|(i, op)| options.operations[..i].contains(op))
        || !options
            .max_mutation_attempts
            .is_some_and(|bound| (1..=128).contains(&bound))
        || !(5..=300).contains(&options.classification_poll_seconds)
        || options.evidence_path.extension().and_then(|v| v.to_str()) != Some("json")
    {
        return Err(DesktopLiveExperimentError::InvalidScope);
    }
    let valid_marker =
        |value: &str| !value.is_empty() && value.is_ascii() && value.len() <= MAX_MOTTO_BYTES;
    if !(valid_marker(&options.control_motto)
        || (options.allow_existing_disposable && options.control_motto.is_empty()))
        || !valid_marker(&options.manual_motto)
        || !valid_marker(&options.motto)
        || options.manual_motto == options.control_motto
        || options.manual_motto == options.motto
        || options.control_motto == options.motto
    {
        return Err(DesktopLiveExperimentError::InvalidScope);
    }
    match options.stage {
        DesktopLiveStage::Immediate => {
            if options.operations.iter().any(|op| {
                matches!(
                    op,
                    DesktopOnlineOperation::AutomaticLevel | DesktopOnlineOperation::AutomaticAct
                )
            }) {
                return Err(DesktopLiveExperimentError::InvalidScope);
            }
            let count: usize = options
                .operations
                .iter()
                .map(|op| DesktopEvidenceCase::required(*op).len())
                .sum::<usize>()
                + usize::from(options.initial_guild_leave);
            if options
                .max_mutation_attempts
                .is_some_and(|bound| bound < count)
            {
                return Err(DesktopLiveExperimentError::InvalidScope);
            }
            if options.operations.contains(&DesktopOnlineOperation::Guild)
                && (options.guild_designation.is_empty()
                    || !options.guild_designation.is_ascii()
                    || options.guild_designation.len() > MAX_GUILD_BYTES
                    || !options
                        .guild_change_designation
                        .as_ref()
                        .is_some_and(|value| {
                            !value.is_empty()
                                && value.is_ascii()
                                && value.len() <= MAX_GUILD_BYTES
                                && value != &options.guild_designation
                                && value != "__gyrognome-invalid__"
                        })
                    || options.guild_designation == "__gyrognome-invalid__")
            {
                return Err(DesktopLiveExperimentError::InvalidScope);
            }
        }
        DesktopLiveStage::Progression => {
            if options.operations.len() != 2
                || !options
                    .operations
                    .contains(&DesktopOnlineOperation::AutomaticLevel)
                || !options
                    .operations
                    .contains(&DesktopOnlineOperation::AutomaticAct)
                || !(MIN_ACTIVE_SECONDS..=MAX_ACTIVE_SECONDS).contains(&options.max_active_seconds)
                || options.max_mutation_attempts.is_some_and(|bound| bound < 2)
            {
                return Err(DesktopLiveExperimentError::InvalidScope);
            }
        }
        DesktopLiveStage::Legacy => return Err(DesktopLiveExperimentError::InvalidScope),
    }
    Ok(contract)
}

pub(super) fn run(
    options: &DesktopLiveExperimentOptions,
) -> Result<(), DesktopLiveExperimentError> {
    let contract = validate_scope(options)?;
    let directory = options
        .experiment_dir
        .canonicalize()
        .map_err(|_| DesktopLiveExperimentError::InvalidExperimentDirectory)?;
    if directory.starts_with(env!("CARGO_MANIFEST_DIR"))
        || directory
            .ancestors()
            .any(|ancestor| ancestor.join(".git").exists())
        || fs::symlink_metadata(&options.experiment_dir)
            .map_err(|_| DesktopLiveExperimentError::PrivateState)?
            .file_type()
            .is_symlink()
        || fs::metadata(&directory)
            .map_err(|_| DesktopLiveExperimentError::PrivateState)?
            .permissions()
            .mode()
            & 0o077
            != 0
    {
        return Err(DesktopLiveExperimentError::InvalidExperimentDirectory);
    }
    for entry in fs::read_dir(&directory).map_err(|_| DesktopLiveExperimentError::PrivateState)? {
        let entry = entry.map_err(|_| DesktopLiveExperimentError::PrivateState)?;
        let metadata = fs::symlink_metadata(entry.path())
            .map_err(|_| DesktopLiveExperimentError::PrivateState)?;
        if metadata.file_type().is_symlink()
            || (metadata.is_file() && metadata.permissions().mode() & 0o077 != 0)
        {
            return Err(DesktopLiveExperimentError::PrivateState);
        }
    }
    let (save_path, backup_path) = select_save_files(&directory)?;
    if backup_path
        .as_ref()
        .is_some_and(|path| path.file_stem() != save_path.file_stem())
    {
        return Err(DesktopLiveExperimentError::InvalidExperimentDirectory);
    }
    if options
        .evidence_path
        .file_name()
        .and_then(|value| value.to_str())
        == Some(CHECKPOINT)
    {
        return Err(DesktopLiveExperimentError::InvalidScope);
    }
    let source_bytes =
        fs::read(&save_path).map_err(|_| DesktopLiveExperimentError::PrivateState)?;
    let source_hash = sha256(&source_bytes);
    let save = match crate::save::import_supported_bytes(&source_bytes)
        .map_err(|_| DesktopLiveExperimentError::UnsupportedSave)?
    {
        ImportedSave::Desktop(save) => save,
        ImportedSave::Browser(_) => return Err(DesktopLiveExperimentError::UnsupportedSave),
    };
    let state = DesktopCanonicalState::from(&save);
    contract
        .validate(
            &save.private.endpoint,
            save.private.passkey,
            &save.private.account,
            &save.private.password,
            desktop_request_text_is_ascii(&state, &[]),
        )
        .map_err(|_| DesktopLiveExperimentError::ContractMismatch)?;
    validate_source_adaptations(options, &save)?;
    if save.private.realm != contract.realm
        || save.game_style != 3
        || (!save.profile.guild.is_empty() && !options.initial_guild_leave)
        || (save.profile.guild.is_empty() && options.initial_guild_leave)
        || save.profile.motto != options.control_motto
    {
        return Err(DesktopLiveExperimentError::ContractMismatch);
    }
    validate_fresh(&save)?;
    let name = trait_value(&save, "Name")?.to_owned();
    let scope = source_scope(options, contract, &save, &source_hash)?;
    let checkpoint_path = directory.join(CHECKPOINT);
    let checkpoint = if checkpoint_path.exists() {
        let checkpoint: StageCheckpoint = serde_json::from_slice(
            &fs::read(&checkpoint_path).map_err(|_| DesktopLiveExperimentError::PrivateState)?,
        )
        .map_err(|_| DesktopLiveExperimentError::PrivateState)?;
        validate_checkpoint_scope(&checkpoint, &scope, options)?;
        checkpoint
    } else {
        if options.confirm_preparation_reconciliation || options.confirm_operator_accepted_join {
            return Err(DesktopLiveExperimentError::InvalidScope);
        }
        StageCheckpoint {
            scope_sha256: scope,
            callback: DesktopCallbackCheckpoint {
                state,
                random: DesktopRandomState(
                    OsRandom::open()
                        .map_err(|_| DesktopLiveExperimentError::PrivateState)?
                        .next_u32()
                        .map_err(|_| DesktopLiveExperimentError::PrivateState)?,
                ),
            },
            active_milliseconds: 0,
            attempts: vec![],
            observations: vec![],
            pending_callback: false,
            last_public_cells: None,
            preparation_recovery: None,
            operator_accepted_join: None,
        }
    };
    let mut context = Context {
        options,
        contract,
        save,
        name,
        checkpoint_path,
        checkpoint,
        transport: &NativeStageTransport,
    };
    if context.checkpoint.operator_accepted_join.is_some() {
        operator_join_binding(&context)?;
    }
    if !checkpoint_can_resume(&context.checkpoint) {
        if options.confirm_operator_accepted_join {
            operator_join_intent(&context)?;
        } else {
            preparation_recovery_intent(&context)?;
        }
    }
    if options.validate_only {
        println!(
            "disposable desktop save, checkpoint, and explicit realm/stage scope validated without network activity"
        );
        return Ok(());
    }
    let stop = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(SIGINT, stop.clone())
        .map_err(|_| DesktopLiveExperimentError::PrivateState)?;
    signal_hook::flag::register(SIGTERM, stop.clone())
        .map_err(|_| DesktopLiveExperimentError::PrivateState)?;
    if !checkpoint_can_resume(&context.checkpoint) {
        let reconciled = if options.confirm_operator_accepted_join {
            reconcile_operator_join(
                &mut context,
                &stop,
                Instant::now() + Duration::from_secs(60),
            )
        } else {
            reconcile_preparation(&mut context, &stop)
        };
        if let Err(error) = reconciled {
            write_status(&context, "reconciliation-inconclusive", false)?;
            return Err(error);
        }
    }
    let baseline = observe(&context, Instant::now() + observation_bound(options))?;
    validate_baseline(&context, &baseline)?;
    persist(&context)?;
    let result = match options.stage {
        DesktopLiveStage::Immediate => immediate(&mut context, &stop),
        DesktopLiveStage::Progression => progression(&mut context, &stop),
        DesktopLiveStage::Legacy => Err(DesktopLiveExperimentError::InvalidScope),
    };
    if let Err(error) = result {
        write_status(&context, "inconclusive", false)?;
        return Err(error);
    }
    let final_row = observe(&context, Instant::now() + observation_bound(options))?;
    if final_row.1 != Some(None) || validate_baseline(&context, &final_row).is_err() {
        write_status(&context, "cleanup-unconfirmed", false)?;
        return Err(DesktopLiveExperimentError::Cleanup);
    }
    // Validate prospective records before deleting inputs; publish only after cleanup.
    let records = match evidence_records(&context, true) {
        Ok(records) => records,
        Err(error) => {
            write_status(&context, "inconclusive", false)?;
            return Err(error);
        }
    };
    write_status(&context, "cleanup-pending", false)?;
    if sha256(&fs::read(&save_path).map_err(|_| DesktopLiveExperimentError::Cleanup)?)
        != source_hash
    {
        return Err(DesktopLiveExperimentError::Cleanup);
    }
    cleanup_files(&context.checkpoint_path, &save_path, backup_path.as_deref())?;
    let output = serde_json::to_string_pretty(&records)
        .map_err(|_| DesktopLiveExperimentError::PrivateState)?;
    atomic_write(&options.evidence_path, output.as_bytes())?;
    if context.checkpoint.operator_accepted_join.is_some() {
        println!(
            "guild production evidence excluded: operator acceptance has no original join response fingerprint"
        );
    }
    println!("scoped desktop stage completed with validated credential-free operation records");
    Ok(())
}

fn validate_source_adaptations(
    options: &DesktopLiveExperimentOptions,
    save: &DesktopValidatedSave,
) -> Result<(), DesktopLiveExperimentError> {
    validate_scope(options)?;
    let adaptations = DesktopImportMetadata::from_validated(&save.adaptations)
        .provenance
        .adaptations;
    if options.allow_quest_placeholder {
        if adaptations != [crate::compatibility::DesktopAdaptation::LegacyQuestPlaceholder]
            || !matches!(save.quest, crate::desktop_save::DesktopQuestMarker::LegacyPlaceholder { index }
                if index < crate::desktop_rules::bundled().tables.monsters.len())
        {
            return Err(DesktopLiveExperimentError::InvalidScope);
        }
    } else if !adaptations.is_empty() {
        return Err(DesktopLiveExperimentError::InvalidScope);
    }
    Ok(())
}

fn source_scope(
    options: &DesktopLiveExperimentOptions,
    contract: DesktopRealmContract,
    save: &DesktopValidatedSave,
    source_hash: &str,
) -> Result<String, DesktopLiveExperimentError> {
    let state = DesktopCanonicalState::from(save);
    let name = trait_value(save, "Name")?;
    let original = sha256(
        &serde_json::to_vec(&(
            contract.realm,
            options.stage,
            &options.operations,
            options.max_mutation_attempts,
            options.max_active_seconds,
            options.classification_poll_seconds,
            &options.control_motto,
            &options.manual_motto,
            &options.motto,
            &options.guild_designation,
            &options.guild_change_designation,
            name,
            source_hash,
            &state,
            options.allow_existing_disposable,
            options.initial_guild_leave,
        ))
        .map_err(|_| DesktopLiveExperimentError::PrivateState)?,
    );
    if !options.allow_quest_placeholder {
        return Ok(original);
    }
    let adaptations = DesktopImportMetadata::from_validated(&save.adaptations)
        .provenance
        .adaptations;
    Ok(sha256(
        &serde_json::to_vec(&(
            original,
            options.allow_quest_placeholder,
            adaptations,
            &save.quest,
        ))
        .map_err(|_| DesktopLiveExperimentError::PrivateState)?,
    ))
}

fn validate_baseline(
    context: &Context<'_>,
    baseline: &(Vec<String>, Option<Option<String>>),
) -> Result<(), DesktopLiveExperimentError> {
    let initial = report(
        &context.checkpoint.callback.state,
        DesktopReportOperation::Manual,
        context.contract.realm,
        &context.checkpoint.callback.state.profile.motto,
        context.save.private.passkey,
        &DesktopAccountAuthentication::new(
            &context.save.private.account,
            &context.save.private.password,
        ),
    )
    .map_err(|_| DesktopLiveExperimentError::Construction)?;
    let expected = context
        .checkpoint
        .last_public_cells
        .clone()
        .unwrap_or(report_cells(&initial)?);
    let guild = &context.checkpoint.callback.state.profile.guild;
    if baseline.0 != expected
        || (context.checkpoint.attempts.is_empty()
            && baseline.0.get(8) != Some(&context.options.control_motto))
        || !baseline
            .1
            .as_ref()
            .is_some_and(|value| public_guild_matches(value.as_deref(), guild))
    {
        return Err(DesktopLiveExperimentError::Inconclusive);
    }
    Ok(())
}

fn observation_bound(options: &DesktopLiveExperimentOptions) -> Duration {
    Duration::from_secs(options.classification_poll_seconds)
}

fn checkpoint_can_resume(checkpoint: &StageCheckpoint) -> bool {
    let recovered = usize::from(checkpoint.preparation_recovery.is_some());
    let accepted = usize::from(checkpoint.operator_accepted_join.is_some());
    !checkpoint.pending_callback
        && checkpoint.attempts.len() == checkpoint.observations.len() + recovered + accepted
        && checkpoint
            .preparation_recovery
            .as_ref()
            .is_none_or(|recovery| {
                checkpoint.attempts.first() == Some(&recovery.intent_sha256)
                    && recovery.within_bound()
                    && checkpoint
                        .observations
                        .iter()
                        .all(|value| value.case.is_some())
            })
        && checkpoint
            .operator_accepted_join
            .as_ref()
            .is_none_or(|acceptance| {
                checkpoint.preparation_recovery.is_some()
                    && checkpoint.active_milliseconds == 0
                    && (5..=8).contains(&checkpoint.attempts.len())
                    && acceptance.no_competing_join_request_confirmed
                    && acceptance.reconciliation.within_bound()
                    && checkpoint.attempts.get(4) == Some(&acceptance.reconciliation.intent_sha256)
                    && checkpoint.observations.iter().map(|value| value.case).eq([
                        DesktopEvidenceCase::ManualConsumed,
                        DesktopEvidenceCase::MottoSet,
                        DesktopEvidenceCase::MottoCleared,
                        DesktopEvidenceCase::GuildChanged,
                        DesktopEvidenceCase::GuildRejected,
                        DesktopEvidenceCase::GuildLeft,
                    ]
                    .into_iter()
                    .take(checkpoint.observations.len())
                    .map(Some))
            })
        && checkpoint
            .attempts
            .iter()
            .enumerate()
            .filter(|(index, _)| !(*index == 0 && recovered == 1 || *index == 4 && accepted == 1))
            .map(|(_, attempt)| attempt)
            .zip(&checkpoint.observations)
            .all(|(attempt, observation)| attempt == &observation.intent_sha256)
}

fn validate_checkpoint_scope(
    checkpoint: &StageCheckpoint,
    scope: &str,
    options: &DesktopLiveExperimentOptions,
) -> Result<(), DesktopLiveExperimentError> {
    if checkpoint.scope_sha256 != scope
        || (checkpoint.operator_accepted_join.is_some() && !options.confirm_operator_accepted_join)
        || (!checkpoint_can_resume(checkpoint)
            && !options.confirm_preparation_reconciliation
            && !options.confirm_operator_accepted_join)
    {
        return Err(DesktopLiveExperimentError::Inconclusive);
    }
    Ok(())
}

fn unix_milliseconds() -> Result<u64, DesktopLiveExperimentError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|value| u64::try_from(value.as_millis()).ok())
        .ok_or(DesktopLiveExperimentError::PrivateState)
}

fn reconcile_preparation(
    context: &mut Context<'_>,
    stop: &AtomicBool,
) -> Result<(), DesktopLiveExperimentError> {
    reconcile_preparation_until(context, stop, Instant::now() + Duration::from_secs(60))
}

fn reconcile_preparation_until(
    context: &mut Context<'_>,
    stop: &AtomicBool,
    deadline: Instant,
) -> Result<(), DesktopLiveExperimentError> {
    let (intent, expected) = preparation_recovery_intent(context)?;
    let authorized_at = unix_milliseconds()?;
    if Instant::now() >= deadline {
        return Err(DesktopLiveExperimentError::Inconclusive);
    }
    if stop.load(Ordering::Relaxed) {
        return Err(DesktopLiveExperimentError::Interrupted);
    }
    let observation = observe(context, deadline)?;
    let observed_at = unix_milliseconds()?;
    if observation.0 != expected
        || observation.1 != Some(None)
        || Instant::now() > deadline
        || observed_at < authorized_at
        || observed_at - authorized_at > 60_000
    {
        return Err(DesktopLiveExperimentError::Inconclusive);
    }
    context.checkpoint.preparation_recovery = Some(PreparationRecovery {
        intent_sha256: intent,
        authorized_at_unix_milliseconds: authorized_at,
        observed_at_unix_milliseconds: observed_at,
        read_only_window_seconds: 60,
    });
    context.checkpoint.callback.state.profile.guild.clear();
    context.checkpoint.last_public_cells = Some(expected);
    persist(context)
}

fn preparation_recovery_intent(
    context: &Context<'_>,
) -> Result<(String, Vec<String>), DesktopLiveExperimentError> {
    validate_scope(context.options)?;
    if !context.options.confirm_preparation_reconciliation
        || context.checkpoint.preparation_recovery.is_some()
        || context.checkpoint.attempts.len() != 1
        || !context.checkpoint.observations.is_empty()
        || context.checkpoint.pending_callback
        || context.checkpoint.active_milliseconds != 0
        || context.checkpoint.last_public_cells.is_some()
        || context.checkpoint.callback.state != DesktopCanonicalState::from(&context.save)
    {
        return Err(DesktopLiveExperimentError::Inconclusive);
    }
    let authentication = DesktopAccountAuthentication::new(
        &context.save.private.account,
        &context.save.private.password,
    );
    let request = guild_request(
        &context.checkpoint.callback.state,
        context.contract.realm,
        "",
        context.save.private.passkey,
        &authentication,
    )
    .map_err(|_| DesktopLiveExperimentError::Construction)?;
    let intent = request_intent_sha256("initial-guild-leave", &request.encoded_query());
    if context.checkpoint.attempts[0] != intent {
        return Err(DesktopLiveExperimentError::Inconclusive);
    }
    let expected = report_cells(
        &report(
            &context.checkpoint.callback.state,
            DesktopReportOperation::Manual,
            context.contract.realm,
            &context.save.profile.motto,
            context.save.private.passkey,
            &authentication,
        )
        .map_err(|_| DesktopLiveExperimentError::Construction)?,
    )?;
    Ok((intent, expected))
}

fn operator_join_binding(
    context: &Context<'_>,
) -> Result<(String, Vec<String>), DesktopLiveExperimentError> {
    validate_scope(context.options)?;
    if !context.options.confirm_operator_accepted_join
        || context.save.profile.motto != context.options.control_motto
        || context.save.profile.guild.is_empty()
        || context.checkpoint.pending_callback
        || context.checkpoint.active_milliseconds != 0
        || context.checkpoint.observations.len() < 3
        || context
            .checkpoint
            .preparation_recovery
            .as_ref()
            .is_none_or(|value| !value.within_bound())
    {
        return Err(DesktopLiveExperimentError::Inconclusive);
    }
    let authentication = DesktopAccountAuthentication::new(
        &context.save.private.account,
        &context.save.private.password,
    );
    let mut state = DesktopCanonicalState::from(&context.save);
    let preparation = guild_request(
        &state,
        context.contract.realm,
        "",
        context.save.private.passkey,
        &authentication,
    )
    .map_err(|_| DesktopLiveExperimentError::Construction)?;
    let preparation_hash =
        request_intent_sha256("initial-guild-leave", &preparation.encoded_query());
    if context.checkpoint.attempts.first() != Some(&preparation_hash)
        || context
            .checkpoint
            .preparation_recovery
            .as_ref()
            .is_none_or(|value| value.intent_sha256 != preparation_hash)
    {
        return Err(DesktopLiveExperimentError::Inconclusive);
    }
    state.profile.guild.clear();
    let mut expected = vec![];
    for (index, (case, operation, motto)) in [
        (
            DesktopEvidenceCase::ManualConsumed,
            DesktopReportOperation::Manual,
            context.options.manual_motto.as_str(),
        ),
        (
            DesktopEvidenceCase::MottoSet,
            DesktopReportOperation::Motto,
            context.options.motto.as_str(),
        ),
        (
            DesktopEvidenceCase::MottoCleared,
            DesktopReportOperation::Motto,
            "",
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let request = report(
            &state,
            operation,
            context.contract.realm,
            motto,
            context.save.private.passkey,
            &authentication,
        )
        .map_err(|_| DesktopLiveExperimentError::Construction)?;
        let intent = request_intent_sha256(&format!("{case:?}"), &request.encoded_query());
        let observation = &context.checkpoint.observations[index];
        if context.checkpoint.attempts.get(index + 1) != Some(&intent)
            || observation.case != Some(case)
            || observation.intent_sha256 != intent
        {
            return Err(DesktopLiveExperimentError::Inconclusive);
        }
        expected = report_cells(&request)?;
        state.profile.motto = motto.to_owned();
    }
    let mut stored = context.checkpoint.callback.state.clone();
    stored.profile.guild.clear();
    if stored != state || context.checkpoint.last_public_cells.as_ref() != Some(&expected) {
        return Err(DesktopLiveExperimentError::Inconclusive);
    }
    let request = guild_request(
        &state,
        context.contract.realm,
        &context.options.guild_designation,
        context.save.private.passkey,
        &authentication,
    )
    .map_err(|_| DesktopLiveExperimentError::Construction)?;
    let intent = request_intent_sha256("GuildJoined", &request.encoded_query());
    if context.checkpoint.attempts.get(4) != Some(&intent) {
        return Err(DesktopLiveExperimentError::Inconclusive);
    }
    Ok((intent, expected))
}

fn operator_join_intent(
    context: &Context<'_>,
) -> Result<(String, Vec<String>), DesktopLiveExperimentError> {
    if context.checkpoint.operator_accepted_join.is_some()
        || context.checkpoint.attempts.len() != 5
        || context.checkpoint.observations.len() != 3
        || !context.checkpoint.callback.state.profile.guild.is_empty()
    {
        return Err(DesktopLiveExperimentError::Inconclusive);
    }
    operator_join_binding(context)
}

fn reconcile_operator_join(
    context: &mut Context<'_>,
    stop: &AtomicBool,
    deadline: Instant,
) -> Result<(), DesktopLiveExperimentError> {
    let (intent, expected) = operator_join_intent(context)?;
    let authorized_at = unix_milliseconds()?;
    if Instant::now() >= deadline {
        return Err(DesktopLiveExperimentError::Inconclusive);
    }
    if stop.load(Ordering::Relaxed) {
        return Err(DesktopLiveExperimentError::Interrupted);
    }
    let observed = observe(context, deadline)?;
    let observed_at = unix_milliseconds()?;
    let reconciliation = PreparationRecovery {
        intent_sha256: intent,
        authorized_at_unix_milliseconds: authorized_at,
        observed_at_unix_milliseconds: observed_at,
        read_only_window_seconds: 60,
    };
    if Instant::now() > deadline
        || !reconciliation.within_bound()
        || observed.0 != expected
        || !observed.1.as_ref().is_some_and(|guild| {
            public_guild_matches(guild.as_deref(), &context.options.guild_designation)
        })
    {
        return Err(DesktopLiveExperimentError::Inconclusive);
    }
    let guild = observed
        .1
        .flatten()
        .ok_or(DesktopLiveExperimentError::Inconclusive)?;
    context.checkpoint.operator_accepted_join = Some(OperatorAcceptedJoin {
        reconciliation,
        no_competing_join_request_confirmed: context.options.confirm_no_competing_join,
    });
    context.checkpoint.callback.state.profile.guild = guild;
    persist(context)
}

fn observe(
    context: &Context<'_>,
    deadline: Instant,
) -> Result<(Vec<String>, Option<Option<String>>), DesktopLiveExperimentError> {
    let page = context
        .transport
        .page(context.contract, &context.name, deadline)?;
    if page.len() > 256 * 1024 || Instant::now() > deadline {
        return Err(DesktopLiveExperimentError::Inconclusive);
    }
    let observed =
        parse_observation(&page, &context.name).ok_or(DesktopLiveExperimentError::Inconclusive)?;
    match observed.classification {
        PublicClassification::Normal => {
            let cells = observed
                .cells
                .ok_or(DesktopLiveExperimentError::Inconclusive)?;
            if context.contract.realm == "Pemptus" {
                let (strict_cells, guild) = pemptus_public_row(&page, &context.name)
                    .ok_or(DesktopLiveExperimentError::Inconclusive)?;
                if strict_cells != cells {
                    return Err(DesktopLiveExperimentError::Inconclusive);
                }
                Ok((cells, Some(guild)))
            } else {
                Ok((
                    cells,
                    public_guild_for_realm(context.contract.realm, &page, &context.name),
                ))
            }
        }
        PublicClassification::Cheater => Err(DesktopLiveExperimentError::Cheater),
        PublicClassification::Unknown => Err(DesktopLiveExperimentError::Inconclusive),
    }
}

fn poll(
    context: &Context<'_>,
    expected: &[String],
    guild: Option<&str>,
    stop: &AtomicBool,
) -> Result<Option<String>, DesktopLiveExperimentError> {
    let deadline = Instant::now() + observation_bound(context.options);
    loop {
        if stop.load(Ordering::Relaxed) {
            return Err(DesktopLiveExperimentError::Interrupted);
        }
        let observed = observe(context, deadline)?;
        if observed.0 == expected
            && guild.is_none_or(|guild| {
                observed
                    .1
                    .as_ref()
                    .is_some_and(|value| public_guild_matches(value.as_deref(), guild))
            })
        {
            return Ok(observed.1.flatten());
        }
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Err(DesktopLiveExperimentError::Inconclusive);
        }
        thread::sleep(PUBLIC_POLL_INTERVAL.min(remaining));
    }
}

fn persist(context: &Context<'_>) -> Result<(), DesktopLiveExperimentError> {
    atomic_write(
        &context.checkpoint_path,
        &serde_json::to_vec(&context.checkpoint)
            .map_err(|_| DesktopLiveExperimentError::PrivateState)?,
    )
}

fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), DesktopLiveExperimentError> {
    let temporary = path.with_extension("json.tmp");
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&temporary)
        .map_err(|_| DesktopLiveExperimentError::PrivateState)?;
    file.write_all(bytes)
        .and_then(|_| file.sync_all())
        .map_err(|_| DesktopLiveExperimentError::PrivateState)?;
    fs::rename(&temporary, path).map_err(|_| DesktopLiveExperimentError::PrivateState)
}

fn cleanup_files(
    checkpoint: &Path,
    save: &Path,
    backup: Option<&Path>,
) -> Result<(), DesktopLiveExperimentError> {
    cleanup_files_with(checkpoint, save, backup, |path| fs::remove_file(path))
}

fn cleanup_files_with(
    checkpoint: &Path,
    save: &Path,
    backup: Option<&Path>,
    mut remove: impl FnMut(&Path) -> std::io::Result<()>,
) -> Result<(), DesktopLiveExperimentError> {
    for path in [Some(checkpoint), Some(save), backup].into_iter().flatten() {
        let metadata =
            fs::symlink_metadata(path).map_err(|_| DesktopLiveExperimentError::Cleanup)?;
        if !metadata.is_file() || metadata.file_type().is_symlink() {
            return Err(DesktopLiveExperimentError::Cleanup);
        }
    }
    // Retain the intent ledger until every reusable source has been removed.
    for path in [Some(save), backup, Some(checkpoint)].into_iter().flatten() {
        remove(path).map_err(|_| DesktopLiveExperimentError::Cleanup)?;
    }
    Ok(())
}

fn complete(context: &Context<'_>, case: DesktopEvidenceCase) -> bool {
    context
        .checkpoint
        .observations
        .iter()
        .any(|value| value.case == Some(case))
}

fn report_cells(
    request: &crate::desktop_protocol::ConstructedDesktopReport,
) -> Result<Vec<String>, DesktopLiveExperimentError> {
    request
        .public_cells()
        .ok_or(DesktopLiveExperimentError::Construction)
}

struct Intent {
    case: Option<DesktopEvidenceCase>,
    query: String,
    expected_cells: Vec<String>,
    guild: Option<String>,
}

fn attempt(
    context: &mut Context<'_>,
    intent: Intent,
    stop: &AtomicBool,
) -> Result<(), DesktopLiveExperimentError> {
    if stop.load(Ordering::Relaxed) {
        return Err(DesktopLiveExperimentError::Interrupted);
    }
    let label = intent.case.map_or_else(
        || "initial-guild-leave".to_owned(),
        |case| format!("{case:?}"),
    );
    let hash = request_intent_sha256(&label, &intent.query);
    if context.checkpoint.attempts.contains(&hash)
        || context
            .options
            .max_mutation_attempts
            .is_none_or(|bound| context.checkpoint.attempts.len() >= bound)
    {
        return Err(DesktopLiveExperimentError::InvalidScope);
    }
    context.checkpoint.attempts.push(hash.clone());
    persist(context)?;
    let target =
        resolve_verified_desktop_endpoint(context.contract.realm, context.contract.saved_endpoint)
            .map_err(|_| DesktopLiveExperimentError::ContractMismatch)?;
    let credentials = DesktopTransportCredentials::new(
        &context.save.private.account,
        &context.save.private.password,
    );
    let response = context
        .transport
        .deliver(&target, &intent.query, &credentials)?;
    if !(200..300).contains(&response.status) {
        return Err(DesktopLiveExperimentError::Delivery);
    }
    let key = context.save.private.passkey.to_string();
    let authorization = credentials
        .authorization_header(target.credential_mode())
        .map_err(|_| DesktopLiveExperimentError::ContractMismatch)?;
    let submitted_guild = url::form_urlencoded::parse(intent.query.as_bytes())
        .find(|(key, _)| key == "guild")
        .map(|(_, value)| value.into_owned());
    let fingerprint = DesktopGuildFingerprintValues {
        character_name: &context.name,
        account: &context.save.private.account,
        password: &context.save.private.password,
        authorization: authorization.as_deref().unwrap_or(""),
        passkey: &key,
        prior_guild: &context.checkpoint.callback.state.profile.guild,
        submitted_guild: submitted_guild.as_deref().unwrap_or(""),
    }
    .fingerprint(&response.body);
    let observed_guild = poll(
        context,
        &intent.expected_cells,
        intent.guild.as_deref(),
        stop,
    )?;
    context.checkpoint.observations.push(StageObservation {
        intent_sha256: hash,
        case: intent.case,
        fingerprint_sha256: fingerprint,
        observed_on: current_utc_date(),
    });
    if let Some(motto) = intent.expected_cells.get(8) {
        context.checkpoint.callback.state.profile.motto = motto.clone();
    }
    if intent.guild.is_some() {
        context.checkpoint.callback.state.profile.guild = observed_guild.unwrap_or_default();
    }
    context.checkpoint.last_public_cells = Some(intent.expected_cells);
    persist(context)
}

fn immediate(
    context: &mut Context<'_>,
    stop: &AtomicBool,
) -> Result<(), DesktopLiveExperimentError> {
    if !checkpoint_can_resume(&context.checkpoint) {
        return Err(DesktopLiveExperimentError::Inconclusive);
    }
    prepare_initial_guild_leave(context, stop)?;
    for operation in context.options.operations.clone() {
        for &case in DesktopEvidenceCase::required(operation) {
            if complete(context, case)
                || (case == DesktopEvidenceCase::GuildJoined
                    && context.checkpoint.operator_accepted_join.is_some())
            {
                continue;
            }
            let baseline = observe(context, Instant::now() + observation_bound(context.options))?;
            validate_baseline(context, &baseline)?;
            let authentication = DesktopAccountAuthentication::new(
                &context.save.private.account,
                &context.save.private.password,
            );
            let (query, expected_cells, guild) = match case {
                DesktopEvidenceCase::ManualConsumed
                | DesktopEvidenceCase::MottoSet
                | DesktopEvidenceCase::MottoCleared => {
                    let (kind, motto) = match case {
                        DesktopEvidenceCase::ManualConsumed => (
                            DesktopReportOperation::Manual,
                            context.options.manual_motto.as_str(),
                        ),
                        DesktopEvidenceCase::MottoSet => (
                            DesktopReportOperation::Motto,
                            context.options.motto.as_str(),
                        ),
                        _ => (DesktopReportOperation::Motto, ""),
                    };
                    let request = report(
                        &context.checkpoint.callback.state,
                        kind,
                        context.contract.realm,
                        motto,
                        context.save.private.passkey,
                        &authentication,
                    )
                    .map_err(|_| DesktopLiveExperimentError::Construction)?;
                    let expected = report_cells(&request)?;
                    if expected == baseline.0 {
                        return Err(DesktopLiveExperimentError::Inconclusive);
                    }
                    (request.encoded_query(), expected, None)
                }
                _ => {
                    let submitted = match case {
                        DesktopEvidenceCase::GuildJoined => {
                            context.options.guild_designation.as_str()
                        }
                        DesktopEvidenceCase::GuildChanged => context
                            .options
                            .guild_change_designation
                            .as_deref()
                            .ok_or(DesktopLiveExperimentError::InvalidScope)?,
                        DesktopEvidenceCase::GuildRejected => "__gyrognome-invalid__",
                        DesktopEvidenceCase::GuildLeft => "",
                        _ => return Err(DesktopLiveExperimentError::InvalidScope),
                    };
                    let request = guild_request(
                        &context.checkpoint.callback.state,
                        context.contract.realm,
                        submitted,
                        context.save.private.passkey,
                        &authentication,
                    )
                    .map_err(|_| DesktopLiveExperimentError::Construction)?;
                    let expected_guild = if case == DesktopEvidenceCase::GuildRejected {
                        context.checkpoint.callback.state.profile.guild.clone()
                    } else {
                        submitted.to_owned()
                    };
                    if case != DesktopEvidenceCase::GuildRejected
                        && baseline.1.as_ref().is_some_and(|guild| {
                            public_guild_matches(guild.as_deref(), &expected_guild)
                        })
                    {
                        return Err(DesktopLiveExperimentError::Inconclusive);
                    }
                    (request.encoded_query(), baseline.0, Some(expected_guild))
                }
            };
            attempt(
                context,
                Intent {
                    case: Some(case),
                    query,
                    expected_cells,
                    guild,
                },
                stop,
            )?;
        }
    }
    Ok(())
}

fn prepare_initial_guild_leave(
    context: &mut Context<'_>,
    stop: &AtomicBool,
) -> Result<(), DesktopLiveExperimentError> {
    if !context.options.initial_guild_leave
        || context.checkpoint.preparation_recovery.is_some()
        || context
            .checkpoint
            .observations
            .iter()
            .any(|value| value.case.is_none())
    {
        return Ok(());
    }
    if !checkpoint_can_resume(&context.checkpoint) {
        return Err(DesktopLiveExperimentError::Inconclusive);
    }
    let baseline = observe(context, Instant::now() + observation_bound(context.options))?;
    validate_baseline(context, &baseline)?;
    let guild = &context.checkpoint.callback.state.profile.guild;
    if guild.is_empty()
        || !baseline
            .1
            .as_ref()
            .is_some_and(|value| public_guild_matches(value.as_deref(), guild))
    {
        return Err(DesktopLiveExperimentError::Inconclusive);
    }
    let request = guild_request(
        &context.checkpoint.callback.state,
        context.contract.realm,
        "",
        context.save.private.passkey,
        &DesktopAccountAuthentication::new(
            &context.save.private.account,
            &context.save.private.password,
        ),
    )
    .map_err(|_| DesktopLiveExperimentError::Construction)?;
    attempt(
        context,
        Intent {
            case: None,
            query: request.encoded_query(),
            expected_cells: baseline.0,
            guild: Some(String::new()),
        },
        stop,
    )
}
fn progression(
    context: &mut Context<'_>,
    stop: &AtomicBool,
) -> Result<(), DesktopLiveExperimentError> {
    let mut baseline = Instant::now();
    while !complete(context, DesktopEvidenceCase::LevelAccepted)
        || !complete(context, DesktopEvidenceCase::ActAccepted)
    {
        if stop.load(Ordering::Relaxed) {
            return Err(DesktopLiveExperimentError::Interrupted);
        }
        if context.checkpoint.active_milliseconds >= context.options.max_active_seconds * 1_000 {
            return Err(DesktopLiveExperimentError::BoundExpired);
        }
        thread::sleep(CALLBACK_INTERVAL);
        let elapsed = Instant::now()
            .saturating_duration_since(baseline)
            .as_millis();
        let remaining =
            context.options.max_active_seconds * 1_000 - context.checkpoint.active_milliseconds;
        context.checkpoint.pending_callback = true;
        persist(context)?;
        let mut hooks = SourceDerivedDesktopHooks::traced();
        let result = context
            .checkpoint
            .callback
            .apply_progression_callback(
                i64::try_from(elapsed.min(u128::from(remaining)))
                    .map_err(|_| DesktopLiveExperimentError::Progression)?,
                &mut hooks,
            )
            .map_err(|_| DesktopLiveExperimentError::Progression)?;
        context.checkpoint.active_milliseconds += result.credited_milliseconds;
        persist(context)?;
        for snapshot in hooks.reports() {
            let case = match snapshot.trigger {
                DesktopReportTrigger::Level => DesktopEvidenceCase::LevelAccepted,
                DesktopReportTrigger::Act => DesktopEvidenceCase::ActAccepted,
            };
            let request = report_for_snapshot(
                snapshot,
                context.contract.realm,
                &context.checkpoint.callback.state.profile.motto,
                context.save.private.passkey,
                &DesktopAccountAuthentication::new(
                    &context.save.private.account,
                    &context.save.private.password,
                ),
            )
            .map_err(|_| DesktopLiveExperimentError::Construction)?;
            let expected = report_cells(&request)?;
            let previous = observe(context, Instant::now() + observation_bound(context.options))?;
            if previous.0 == expected {
                return Err(DesktopLiveExperimentError::Inconclusive);
            }
            attempt(
                context,
                Intent {
                    case: Some(case),
                    query: request.encoded_query(),
                    expected_cells: expected,
                    guild: None,
                },
                stop,
            )?;
        }
        context.checkpoint.pending_callback = false;
        persist(context)?;
        baseline = Instant::now();
    }
    Ok(())
}

fn evidence_records(
    context: &Context<'_>,
    cleanup_complete: bool,
) -> Result<BTreeMap<String, serde_json::Value>, DesktopLiveExperimentError> {
    if !checkpoint_can_resume(&context.checkpoint)
        || context
            .checkpoint
            .attempts
            .iter()
            .enumerate()
            .any(|(index, intent)| context.checkpoint.attempts[..index].contains(intent))
        || context
            .checkpoint
            .observations
            .iter()
            .enumerate()
            .any(|(index, value)| {
                value.case.is_some_and(|case| {
                    context.options.stage != DesktopLiveStage::Progression
                        || !matches!(
                            case,
                            DesktopEvidenceCase::LevelAccepted | DesktopEvidenceCase::ActAccepted
                        )
                }) && context.checkpoint.observations[..index]
                    .iter()
                    .any(|prior| prior.case == value.case)
            })
    {
        return Err(DesktopLiveExperimentError::Inconclusive);
    }
    if context.checkpoint.operator_accepted_join.is_some()
        && (![
            DesktopEvidenceCase::GuildChanged,
            DesktopEvidenceCase::GuildRejected,
            DesktopEvidenceCase::GuildLeft,
        ]
        .into_iter()
        .all(|case| complete(context, case))
            || !context.checkpoint.callback.state.profile.guild.is_empty())
    {
        return Err(DesktopLiveExperimentError::Inconclusive);
    }
    let fingerprint = |case| {
        context
            .checkpoint
            .observations
            .iter()
            .find(|value| value.case == Some(case))
            .map(|value| value.fingerprint_sha256.clone())
            .ok_or(DesktopLiveExperimentError::Inconclusive)
    };
    let mut records = BTreeMap::new();
    for &operation in &context.options.operations {
        if operation == DesktopOnlineOperation::Guild
            && context.checkpoint.operator_accepted_join.is_some()
        {
            continue;
        }
        let observed_on = context
            .checkpoint
            .observations
            .iter()
            .filter(|value| {
                value
                    .case
                    .is_some_and(|case| DesktopEvidenceCase::required(operation).contains(&case))
            })
            .map(|value| value.observed_on.as_str())
            .min()
            .ok_or(DesktopLiveExperimentError::Inconclusive)?;
        if observed_on.len() != 10 || !observed_on.is_ascii() {
            return Err(DesktopLiveExperimentError::Inconclusive);
        }
        let next_year = observed_on[..4]
            .parse::<u32>()
            .map_err(|_| DesktopLiveExperimentError::PrivateState)?
            + 1;
        let anniversary = if &observed_on[4..] == "-02-29" {
            "-02-28"
        } else {
            &observed_on[4..]
        };
        let guild_fingerprints = if operation == DesktopOnlineOperation::Guild {
            Some(DesktopGuildFingerprints {
                normalization: DESKTOP_RESPONSE_FINGERPRINT_VERSION.to_owned(),
                join: fingerprint(DesktopEvidenceCase::GuildJoined)?,
                change: Some(fingerprint(DesktopEvidenceCase::GuildChanged)?),
                rejected: fingerprint(DesktopEvidenceCase::GuildRejected)?,
                leave: fingerprint(DesktopEvidenceCase::GuildLeft)?,
            })
        } else {
            None
        };
        let accepted_cases = DesktopEvidenceCase::required(operation)
            .iter()
            .filter(|case| complete(context, **case))
            .copied()
            .collect();
        let encoded = encode_operation_evidence(DesktopOperationEvidenceObservation {
            contract: context.contract,
            operation,
            adaptations: DesktopImportMetadata::from_validated(&context.save.adaptations)
                .provenance
                .adaptations,
            accepted_cases,
            guild_fingerprints,
            observed_on: observed_on.to_owned(),
            valid_through: format!("{next_year}{anniversary}"),
            cleanup_complete,
            normal_classification: true,
        })
        .map_err(|_| DesktopLiveExperimentError::Inconclusive)?;
        records.insert(
            operation.label().replace(' ', "-"),
            serde_json::from_str(&encoded).map_err(|_| DesktopLiveExperimentError::PrivateState)?,
        );
    }
    Ok(records)
}

fn write_status(
    context: &Context<'_>,
    status: &str,
    cleanup_complete: bool,
) -> Result<(), DesktopLiveExperimentError> {
    let content = serde_json::to_vec_pretty(&serde_json::json!({
        "format": "gyrognome-desktop-stage-diagnostic/v1",
        "status": status, "realm": context.contract.realm,
        "stage": context.options.stage, "mutationAttempts": context.checkpoint.attempts.len(),
        "cleanupComplete": cleanup_complete, "productionEligibilityEnabled": false,
        "guildEvidenceExcluded": context.checkpoint.operator_accepted_join.is_some(),
        "guildEvidenceExclusionReason": context.checkpoint.operator_accepted_join.as_ref()
            .map(|_| "operator-accepted-join-without-original-response-fingerprint"),
    }))
    .map_err(|_| DesktopLiveExperimentError::PrivateState)?;
    atomic_write(&context.options.evidence_path, &content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::desktop_evidence::{DesktopEvidenceExpectation, validate_desktop_evidence};
    use std::cell::{Cell, RefCell};

    struct FakeStageTransport {
        cells: RefCell<Vec<String>>,
        guild: RefCell<String>,
        calls: Cell<usize>,
        checkpoint: PathBuf,
        unchanged: bool,
        pages: Cell<usize>,
        fail_page_at_call: Cell<usize>,
        page_delay_milliseconds: Cell<u64>,
        abnormal: Cell<bool>,
    }

    impl StageTransport for FakeStageTransport {
        fn page(
            &self,
            _: DesktopRealmContract,
            _: &str,
            _: Instant,
        ) -> Result<String, DesktopLiveExperimentError> {
            self.pages.set(self.pages.get() + 1);
            if (self.unchanged && self.pages.get() > 2)
                || (self.fail_page_at_call.get() != 0
                    && self.calls.get() == self.fail_page_at_call.get())
            {
                return Err(DesktopLiveExperimentError::Inconclusive);
            }
            thread::sleep(Duration::from_millis(self.page_delay_milliseconds.get()));
            let cells = self.cells.borrow();
            let guild = self.guild.borrow();
            let guild_cell = if guild.is_empty() {
                String::new()
            } else {
                format!("<a href='guilds.php?id=1'>{}</a>", html_escape(&guild))
            };
            Ok(format!(
                "<h1>{}</h1><table><tr><th>Rank<th>Name<th>Race<th>Class<th>Level<th>Prime Stat<th>Plot Stage<th>Prized Item<th>Specialty<th>Motto<th>Guild<tr><td>1{}<td>{guild_cell}</table>",
                if self.abnormal.get() {
                    "Hall of Infamy"
                } else {
                    "Hall of Fame"
                },
                cells
                    .iter()
                    .map(|cell| format!("<td>{}", html_escape(cell)))
                    .collect::<String>()
            ))
        }

        fn deliver(
            &self,
            target: &crate::desktop_transport::VerifiedDesktopEndpoint,
            query: &str,
            credentials: &DesktopTransportCredentials,
        ) -> Result<crate::desktop_transport::DesktopHttpResponse, DesktopLiveExperimentError>
        {
            assert!(
                credentials
                    .authorization_header(target.credential_mode())
                    .unwrap()
                    .is_none()
            );
            let checkpoint: StageCheckpoint =
                serde_json::from_slice(&fs::read(&self.checkpoint).unwrap()).unwrap();
            assert_eq!(
                checkpoint.attempts.len(),
                checkpoint.observations.len()
                    + 1
                    + usize::from(checkpoint.preparation_recovery.is_some())
                    + usize::from(checkpoint.operator_accepted_join.is_some())
            );
            self.calls.set(self.calls.get() + 1);
            let fields: Vec<_> = url::form_urlencoded::parse(query.as_bytes()).collect();
            let field = |name| {
                fields
                    .iter()
                    .find(|(key, _)| key == name)
                    .map(|(_, value)| value.to_string())
            };
            assert_eq!(field("h").as_deref(), Some("Pemptus"));
            let body = if let Some(guild) = field("guild") {
                match guild.as_str() {
                    value if value.eq_ignore_ascii_case("Guild A") || value == "BEERGuild" => {
                        *self.guild.borrow_mut() = if value == "BEERGuild" {
                            "BEERguild".to_owned()
                        } else {
                            "Guild A".to_owned()
                        };
                        b"joined".to_vec()
                    }
                    value if value.eq_ignore_ascii_case("Guild B") || value == "QoD" => {
                        *self.guild.borrow_mut() = if value == "QoD" {
                            "QoD".to_owned()
                        } else {
                            "Guild B".to_owned()
                        };
                        b"changed".to_vec()
                    }
                    "" => {
                        if !self.unchanged {
                            self.guild.borrow_mut().clear();
                        }
                        b"left".to_vec()
                    }
                    _ => b"rejected".to_vec(),
                }
            } else {
                if !self.unchanged {
                    *self.cells.borrow_mut() = crate::desktop_protocol::PUBLIC_REPORT_FIELDS
                        .into_iter()
                        .map(|name| field(name).unwrap_or_default())
                        .collect();
                }
                vec![]
            };
            Ok(crate::desktop_transport::DesktopHttpResponse {
                status: 200,
                redirect: None,
                body,
            })
        }
    }

    fn options(directory: &Path) -> DesktopLiveExperimentOptions {
        DesktopLiveExperimentOptions {
            allow_existing_disposable: false,
            allow_quest_placeholder: false,
            initial_guild_leave: false,
            preparation_reconciliation_seconds: None,
            confirm_preparation_reconciliation: false,
            operator_join_reconciliation_seconds: None,
            confirm_operator_accepted_join: false,
            confirm_no_competing_join: false,
            realm: "Pemptus".to_owned(),
            stage: DesktopLiveStage::Immediate,
            operations: vec![
                DesktopOnlineOperation::ManualBrag,
                DesktopOnlineOperation::Motto,
                DesktopOnlineOperation::Guild,
            ],
            max_mutation_attempts: Some(7),
            manual_motto: "Manual marker".to_owned(),
            guild_change_designation: Some("Guild B".to_owned()),
            confirm_disposable: true,
            confirm_live_submission: true,
            confirm_client_stopped: true,
            confirm_cleanup: true,
            validate_only: false,
            immediate_only: false,
            experiment_dir: directory.to_owned(),
            evidence_path: directory.join("evidence.json"),
            control_motto: "Official control".to_owned(),
            motto: "Motto marker".to_owned(),
            guild_designation: "Guild A".to_owned(),
            max_active_seconds: MAX_ACTIVE_SECONDS,
            classification_poll_seconds: 5,
        }
    }

    fn placeholder_options(directory: &Path) -> DesktopLiveExperimentOptions {
        let mut options = options(directory);
        options.stage = DesktopLiveStage::Progression;
        options.operations = vec![
            DesktopOnlineOperation::AutomaticLevel,
            DesktopOnlineOperation::AutomaticAct,
        ];
        options.allow_quest_placeholder = true;
        options.max_mutation_attempts = Some(32);
        options
    }

    fn placeholder_save() -> DesktopValidatedSave {
        let mut save = crate::pemptus_acceptance_probe::tests::synthetic_save();
        save.adaptations.legacy_quest_placeholder = true;
        save.quest = crate::desktop_save::DesktopQuestMarker::LegacyPlaceholder { index: 1 };
        save
    }

    #[test]
    fn placeholder_admission_requires_exact_source_and_explicit_progression_scope() {
        let directory = TestDirectory::new();
        let mut options = placeholder_options(&directory.0);
        let save = placeholder_save();
        assert!(validate_source_adaptations(&options, &save).is_ok());
        options.allow_quest_placeholder = false;
        assert!(validate_source_adaptations(&options, &save).is_err());
        options.allow_quest_placeholder = true;
        for stage in [DesktopLiveStage::Immediate, DesktopLiveStage::Legacy] {
            options.stage = stage;
            assert!(validate_source_adaptations(&options, &save).is_err());
        }
        options.stage = DesktopLiveStage::Progression;
        options.realm = "Spoltog".to_owned();
        assert!(validate_source_adaptations(&options, &save).is_err());
        options.realm = "Pemptus".to_owned();
        options.confirm_live_submission = false;
        assert!(validate_source_adaptations(&options, &save).is_err());
        options.confirm_live_submission = true;
        options.operations.pop();
        assert!(validate_source_adaptations(&options, &save).is_err());
        options
            .operations
            .push(DesktopOnlineOperation::AutomaticAct);
        assert!(
            validate_source_adaptations(
                &options,
                &crate::pemptus_acceptance_probe::tests::synthetic_save()
            )
            .is_err()
        );
        for marker in [
            crate::desktop_save::DesktopQuestMarker::None,
            crate::desktop_save::DesktopQuestMarker::Value {
                marker: "Unknown".to_owned(),
                index: 1,
            },
            crate::desktop_save::DesktopQuestMarker::LegacyPlaceholder {
                index: crate::desktop_rules::bundled().tables.monsters.len(),
            },
        ] {
            let mut invalid = save.clone();
            invalid.quest = marker;
            assert!(validate_source_adaptations(&options, &invalid).is_err());
        }
        let mut combined = save.clone();
        combined.adaptations.spelling_patch_applied = true;
        assert!(validate_source_adaptations(&options, &combined).is_err());
        combined.adaptations.spelling_patch_applied = false;
        combined.adaptations.legacy_prologue_62 = true;
        assert!(validate_source_adaptations(&options, &combined).is_err());
        assert!(!directory.0.join(CHECKPOINT).exists());
    }

    #[test]
    fn placeholder_source_scope_binds_admission_provenance_index_and_source_without_replay() {
        let directory = TestDirectory::new();
        let mut options = placeholder_options(&directory.0);
        let save = placeholder_save();
        let bound = source_scope(
            &options,
            crate::desktop_contract::PEMPTUS,
            &save,
            "source-a",
        )
        .unwrap();
        options.allow_quest_placeholder = false;
        let different_flag = source_scope(
            &options,
            crate::desktop_contract::PEMPTUS,
            &save,
            "source-a",
        )
        .unwrap();
        assert_ne!(bound, different_flag);
        options.allow_quest_placeholder = true;
        assert_ne!(
            bound,
            source_scope(
                &options,
                crate::desktop_contract::PEMPTUS,
                &save,
                "source-b"
            )
            .unwrap()
        );
        let mut other = save.clone();
        other.quest = crate::desktop_save::DesktopQuestMarker::LegacyPlaceholder { index: 2 };
        assert_ne!(
            bound,
            source_scope(
                &options,
                crate::desktop_contract::PEMPTUS,
                &other,
                "source-a"
            )
            .unwrap()
        );
        other = save.clone();
        other.adaptations.spelling_patch_applied = true;
        assert_ne!(
            bound,
            source_scope(
                &options,
                crate::desktop_contract::PEMPTUS,
                &other,
                "source-a"
            )
            .unwrap()
        );
        other = save.clone();
        other.activity = "Different synthetic state".to_owned();
        assert_ne!(
            bound,
            source_scope(
                &options,
                crate::desktop_contract::PEMPTUS,
                &other,
                "source-a"
            )
            .unwrap()
        );
        let transport = fake(&options, false);
        let mut context = context(&options, &transport);
        context.checkpoint.scope_sha256 = bound.clone();
        assert!(validate_checkpoint_scope(&context.checkpoint, &bound, &options).is_ok());
        assert!(validate_checkpoint_scope(&context.checkpoint, &different_flag, &options).is_err());
        context
            .checkpoint
            .attempts
            .push("unknown-primary".to_owned());
        assert!(!checkpoint_can_resume(&context.checkpoint));
        assert!(validate_checkpoint_scope(&context.checkpoint, &bound, &options).is_err());
        context.checkpoint.attempts.clear();
        context.checkpoint.pending_callback = true;
        assert!(!checkpoint_can_resume(&context.checkpoint));
        assert!(validate_checkpoint_scope(&context.checkpoint, &bound, &options).is_err());
        assert_eq!(transport.calls.get(), 0);
    }

    #[test]
    fn placeholder_records_retain_original_provenance_after_runtime_resolution() {
        let directory = TestDirectory::new();
        let options = placeholder_options(&directory.0);
        let transport = fake(&options, false);
        let mut context = context(&options, &transport);
        context.save = placeholder_save();
        context.checkpoint.callback.state = DesktopCanonicalState::from(&context.save);
        crate::desktop_simulation::resolve_legacy_quest_marker(
            &mut context.checkpoint.callback.state,
        )
        .unwrap();
        for (index, case) in [
            DesktopEvidenceCase::LevelAccepted,
            DesktopEvidenceCase::ActAccepted,
        ]
        .into_iter()
        .enumerate()
        {
            let intent = format!("synthetic-intent-{index}");
            context.checkpoint.attempts.push(intent.clone());
            context.checkpoint.observations.push(StageObservation {
                intent_sha256: intent,
                case: Some(case),
                fingerprint_sha256: "0".repeat(64),
                observed_on: current_utc_date(),
            });
        }
        let records = evidence_records(&context, true).unwrap();
        assert_eq!(records.len(), 2);
        for record in records.values() {
            assert_eq!(
                record["payload"]["importPath"]["adaptations"],
                serde_json::json!(["legacy-quest-placeholder"])
            );
        }
        assert!(evidence_records(&context, false).is_err());
        context.checkpoint.observations.pop();
        context.checkpoint.attempts.pop();
        assert!(evidence_records(&context, true).is_err());
        assert!(context.save.adaptations.legacy_quest_placeholder);
        assert_eq!(transport.calls.get(), 0);
    }

    #[test]
    fn completed_progression_packages_successive_reports_without_callbacks_or_replay() {
        let directory = TestDirectory::new();
        let options = placeholder_options(&directory.0);
        let transport = fake(&options, false);
        let mut context = context(&options, &transport);
        context.save = placeholder_save();
        context.checkpoint.active_milliseconds = 22_429_700;
        for (index, case) in std::iter::repeat_n(DesktopEvidenceCase::LevelAccepted, 12)
            .chain([
                DesktopEvidenceCase::ActAccepted,
                DesktopEvidenceCase::ActAccepted,
            ])
            .enumerate()
        {
            let intent = sha256(format!("synthetic-native-intent-{index}").as_bytes());
            context.checkpoint.attempts.push(intent.clone());
            context.checkpoint.observations.push(StageObservation {
                intent_sha256: intent,
                case: Some(case),
                fingerprint_sha256: "0".repeat(64),
                observed_on: current_utc_date(),
            });
        }
        let before = serde_json::to_vec(&context.checkpoint).unwrap();
        progression(&mut context, &AtomicBool::new(false)).unwrap();
        assert_eq!(serde_json::to_vec(&context.checkpoint).unwrap(), before);
        assert_eq!(transport.calls.get(), 0);
        assert_eq!(transport.pages.get(), 0);
        let records = evidence_records(&context, true).unwrap();
        assert_eq!(records.len(), 2);
        for (operation, case) in [
            (
                DesktopOnlineOperation::AutomaticLevel,
                DesktopEvidenceCase::LevelAccepted,
            ),
            (
                DesktopOnlineOperation::AutomaticAct,
                DesktopEvidenceCase::ActAccepted,
            ),
        ] {
            let record = &records[&operation.label().replace(' ', "-")];
            assert_eq!(
                record["payload"]["acceptedCases"],
                serde_json::json!([case])
            );
            assert_eq!(
                record["payload"]["importPath"]["adaptations"],
                serde_json::json!(["legacy-quest-placeholder"])
            );
        }
        assert!(evidence_records(&context, false).is_err());
        context.checkpoint.pending_callback = true;
        assert!(evidence_records(&context, true).is_err());
        context.checkpoint.pending_callback = false;
        context
            .checkpoint
            .attempts
            .push("unobserved-primary".to_owned());
        assert!(evidence_records(&context, true).is_err());
        context.checkpoint.attempts.pop();
        let duplicate_intent = context.checkpoint.attempts[0].clone();
        context.checkpoint.attempts.push(duplicate_intent.clone());
        context.checkpoint.observations.push(StageObservation {
            intent_sha256: duplicate_intent,
            case: Some(DesktopEvidenceCase::LevelAccepted),
            fingerprint_sha256: "0".repeat(64),
            observed_on: current_utc_date(),
        });
        assert!(checkpoint_can_resume(&context.checkpoint));
        assert!(evidence_records(&context, true).is_err());
    }

    #[test]
    fn immediate_evidence_still_refuses_repeated_primary_cases() {
        let directory = TestDirectory::new();
        let options = options(&directory.0);
        let transport = fake(&options, false);
        let mut context = context(&options, &transport);
        immediate(&mut context, &AtomicBool::new(false)).unwrap();
        assert!(evidence_records(&context, true).is_ok());
        let intent = sha256(b"distinct-synthetic-duplicate-manual");
        context.checkpoint.attempts.push(intent.clone());
        context.checkpoint.observations.push(StageObservation {
            intent_sha256: intent,
            case: Some(DesktopEvidenceCase::ManualConsumed),
            fingerprint_sha256: "0".repeat(64),
            observed_on: current_utc_date(),
        });
        assert!(checkpoint_can_resume(&context.checkpoint));
        assert!(evidence_records(&context, true).is_err());
    }

    fn context<'a>(
        options: &'a DesktopLiveExperimentOptions,
        transport: &'a dyn StageTransport,
    ) -> Context<'a> {
        let mut save = crate::pemptus_acceptance_probe::tests::synthetic_save();
        save.profile.motto = options.control_motto.clone();
        Context {
            name: "Synthetic Hero".to_owned(),
            contract: crate::desktop_contract::PEMPTUS,
            checkpoint_path: options.experiment_dir.join(CHECKPOINT),
            checkpoint: StageCheckpoint {
                scope_sha256: "synthetic".to_owned(),
                callback: DesktopCallbackCheckpoint {
                    state: DesktopCanonicalState::from(&save),
                    random: DesktopRandomState(42),
                },
                active_milliseconds: 0,
                attempts: vec![],
                observations: vec![],
                pending_callback: false,
                last_public_cells: None,
                preparation_recovery: None,
                operator_accepted_join: None,
            },
            options,
            save,
            transport,
        }
    }

    fn fake(options: &DesktopLiveExperimentOptions, unchanged: bool) -> FakeStageTransport {
        let mut save = crate::pemptus_acceptance_probe::tests::synthetic_save();
        save.profile.motto = options.control_motto.clone();
        let request = report(
            &DesktopCanonicalState::from(&save),
            DesktopReportOperation::Manual,
            "Pemptus",
            &save.profile.motto,
            save.private.passkey,
            &DesktopAccountAuthentication::new("", ""),
        )
        .unwrap();
        FakeStageTransport {
            cells: RefCell::new(report_cells(&request).unwrap()),
            guild: RefCell::new(String::new()),
            calls: Cell::new(0),
            checkpoint: options.experiment_dir.join(CHECKPOINT),
            unchanged,
            pages: Cell::new(0),
            fail_page_at_call: Cell::new(0),
            page_delay_milliseconds: Cell::new(0),
            abnormal: Cell::new(false),
        }
    }

    struct TestDirectory(PathBuf);
    impl TestDirectory {
        fn new() -> Self {
            let path =
                Path::new("target/scoped-stage-tests").join(uuid::Uuid::new_v4().to_string());
            fs::create_dir_all(&path).unwrap();
            Self(path)
        }
    }
    impl Drop for TestDirectory {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).unwrap();
        }
    }

    #[test]
    fn rejects_missing_realm_stage_operations_confirmations_and_attempt_bounds() {
        let directory = TestDirectory::new();
        let mut options = options(&directory.0);
        assert!(validate_scope(&options).is_ok());
        options.confirm_client_stopped = false;
        assert!(matches!(
            run(&options),
            Err(DesktopLiveExperimentError::MissingConfirmation)
        ));
        options.confirm_client_stopped = true;
        options.max_mutation_attempts = Some(6);
        assert!(validate_scope(&options).is_err());
        options.max_mutation_attempts = Some(7);
        options.stage = DesktopLiveStage::Legacy;
        assert!(validate_scope(&options).is_err());
        options.stage = DesktopLiveStage::Progression;
        options.operations = vec![
            DesktopOnlineOperation::AutomaticLevel,
            DesktopOnlineOperation::AutomaticAct,
        ];
        for bound in [MIN_ACTIVE_SECONDS, MAX_ACTIVE_SECONDS] {
            options.max_active_seconds = bound;
            assert!(validate_scope(&options).is_ok());
        }
        for bound in [MIN_ACTIVE_SECONDS - 1, MAX_ACTIVE_SECONDS + 1] {
            options.max_active_seconds = bound;
            assert!(validate_scope(&options).is_err());
        }

        options.max_active_seconds = MAX_ACTIVE_SECONDS;
        options.realm = "Unknown".to_owned();
        assert!(validate_scope(&options).is_err());
    }

    #[test]
    fn existing_disposable_scope_requires_immediate_approval_and_preparation_budget() {
        let directory = TestDirectory::new();
        let mut options = options(&directory.0);
        options.control_motto.clear();
        assert!(validate_scope(&options).is_err());
        options.initial_guild_leave = true;
        assert!(validate_scope(&options).is_err());
        options.allow_existing_disposable = true;
        assert!(validate_scope(&options).is_err());
        options.max_mutation_attempts = Some(8);
        assert!(validate_scope(&options).is_ok());
        options.realm = "Spoltog".to_owned();
        assert!(validate_scope(&options).is_err());
        options.realm = "Pemptus".to_owned();
        options.stage = DesktopLiveStage::Progression;
        options.operations = vec![
            DesktopOnlineOperation::AutomaticLevel,
            DesktopOnlineOperation::AutomaticAct,
        ];
        assert!(validate_scope(&options).is_err());
    }

    fn existing_options(directory: &Path) -> DesktopLiveExperimentOptions {
        let mut options = options(directory);
        options.allow_existing_disposable = true;
        options.initial_guild_leave = true;
        options.max_mutation_attempts = Some(8);
        options.control_motto.clear();
        options
    }

    fn operator_options(directory: &Path) -> DesktopLiveExperimentOptions {
        let mut options = existing_options(directory);
        options.preparation_reconciliation_seconds = Some(60);
        options.confirm_preparation_reconciliation = true;
        options.operator_join_reconciliation_seconds = Some(60);
        options.confirm_operator_accepted_join = true;
        options.confirm_no_competing_join = true;
        options.classification_poll_seconds = 60;
        options.guild_designation = "BEERGuild".to_owned();
        options.guild_change_designation = Some("QoD".to_owned());
        options
    }

    fn pending_operator_join(context: &mut Context<'_>, transport: &FakeStageTransport) {
        record_uncertain_preparation(context);
        reconcile_preparation(context, &AtomicBool::new(false)).unwrap();
        transport.fail_page_at_call.set(4);
        assert!(immediate(context, &AtomicBool::new(false)).is_err());
        transport.fail_page_at_call.set(0);
        assert_eq!(context.checkpoint.attempts.len(), 5);
        assert_eq!(context.checkpoint.observations.len(), 3);
        assert_eq!(transport.guild.borrow().as_str(), "BEERguild");
        assert!(!checkpoint_can_resume(&context.checkpoint));
    }

    #[test]
    fn operator_scope_requires_all_confirmations_and_exact_remaining_case_bounds() {
        let directory = TestDirectory::new();
        let mut valid = operator_options(&directory.0);
        assert!(validate_scope(&valid).is_ok());
        valid.preparation_reconciliation_seconds = None;
        valid.confirm_preparation_reconciliation = false;
        assert!(validate_scope(&valid).is_ok());
        for invalid in 0..10 {
            let mut options = operator_options(&directory.0);
            // Operator approval must be checked independently of preparation flags.
            options.preparation_reconciliation_seconds = None;
            options.confirm_preparation_reconciliation = false;
            match invalid {
                0 => options.confirm_operator_accepted_join = false,
                1 => options.confirm_no_competing_join = false,
                2 => options.operator_join_reconciliation_seconds = Some(61),
                3 => options.max_mutation_attempts = Some(9),
                4 => options.classification_poll_seconds = 61,
                5 => options.guild_designation = "Other Guild".to_owned(),
                6 => options.guild_change_designation = Some("Other".to_owned()),
                7 => options.operations.swap(0, 1),
                8 => options.stage = DesktopLiveStage::Progression,
                _ => options.confirm_client_stopped = false,
            }
            assert!(validate_scope(&options).is_err());
        }
    }

    #[test]
    fn operator_join_continuation_is_read_only_skips_completed_cases_and_excludes_guild_evidence() {
        let directory = TestDirectory::new();
        let options = operator_options(&directory.0);
        let transport = fake(&options, false);
        let mut context = context(&options, &transport);
        pending_operator_join(&mut context, &transport);
        let attempts = context.checkpoint.attempts.clone();
        let observations = serde_json::to_vec(&context.checkpoint.observations).unwrap();
        let calls = transport.calls.get();
        reconcile_operator_join(
            &mut context,
            &AtomicBool::new(false),
            Instant::now() + Duration::from_secs(60),
        )
        .unwrap();
        assert_eq!(transport.calls.get(), calls);
        assert_eq!(context.checkpoint.attempts, attempts);
        assert_eq!(
            serde_json::to_vec(&context.checkpoint.observations).unwrap(),
            observations
        );
        assert_eq!(context.checkpoint.callback.state.profile.guild, "BEERguild");
        assert!(!complete(&context, DesktopEvidenceCase::GuildJoined));
        assert!(evidence_records(&context, true).is_err());
        let acceptance = serde_json::to_value(&context.checkpoint.operator_accepted_join).unwrap();
        assert!(!acceptance.to_string().contains("fingerprint"));
        context.checkpoint =
            serde_json::from_slice(&fs::read(&context.checkpoint_path).unwrap()).unwrap();
        assert!(checkpoint_can_resume(&context.checkpoint));
        operator_join_binding(&context).unwrap();
        immediate(&mut context, &AtomicBool::new(false)).unwrap();
        assert_eq!(transport.calls.get() - calls, 3);
        assert_eq!(context.checkpoint.attempts.len(), 8);
        assert_eq!(context.checkpoint.observations.len(), 6);
        assert_eq!(&context.checkpoint.attempts[..5], attempts.as_slice());
        assert_eq!(context.checkpoint.callback.state.profile.guild, "");
        assert!(evidence_records(&context, false).is_err());
        let records = evidence_records(&context, true).unwrap();
        assert_eq!(records.len(), 2);
        assert!(!records.contains_key("guild"));
        for (key, value) in records {
            let operation = if key == "motto" {
                DesktopOnlineOperation::Motto
            } else {
                DesktopOnlineOperation::ManualBrag
            };
            let content = value.to_string();
            crate::fixtures::validate_fixture(Path::new("operator-stage-evidence.json"), &content)
                .unwrap();
            let validated = validate_desktop_evidence(
                &content,
                &DesktopEvidenceExpectation {
                    realm: "Pemptus",
                    saved_endpoint: context.contract.saved_endpoint,
                    verified_https_endpoint: context.contract.https_endpoint,
                    operation,
                    current_date: &current_utc_date(),
                    integrity_sha256: value["integritySha256"].as_str().unwrap(),
                },
            )
            .unwrap();
            assert_eq!(validated.eligibility().operations, [operation]);
        }
        operator_join_binding(&context).unwrap();
        immediate(&mut context, &AtomicBool::new(false)).unwrap();
        assert_eq!(transport.calls.get() - calls, 3);
        write_status(&context, "test-exclusion", true).unwrap();
        let status: serde_json::Value =
            serde_json::from_slice(&fs::read(&options.evidence_path).unwrap()).unwrap();
        assert_eq!(status["guildEvidenceExcluded"], true);
        assert_eq!(status["productionEligibilityEnabled"], false);
        context
            .checkpoint
            .attempts
            .push("unknown-next-case".to_owned());
        assert!(!checkpoint_can_resume(&context.checkpoint));
        assert!(operator_join_intent(&context).is_err());
    }

    #[test]
    fn operator_reconciliation_rejects_changed_intents_state_expiration_and_abnormal_pages() {
        let directory = TestDirectory::new();
        let options = operator_options(&directory.0);
        let transport = fake(&options, false);
        let mut context = context(&options, &transport);
        pending_operator_join(&mut context, &transport);
        let original = serde_json::to_vec(&context.checkpoint).unwrap();
        let calls = transport.calls.get();
        assert!(
            validate_checkpoint_scope(&context.checkpoint, "changed-source-scope", &options)
                .is_err()
        );
        for invalid in 0..9 {
            context.checkpoint = serde_json::from_slice(&original).unwrap();
            match invalid {
                0 => context.checkpoint.attempts[4] = "wrong-primary-intent".to_owned(),
                1 => context
                    .checkpoint
                    .attempts
                    .push("unknown-primary".to_owned()),
                2 => {
                    context.checkpoint.observations.pop();
                }
                3 => context.checkpoint.active_milliseconds = 1,
                4 => context.checkpoint.pending_callback = true,
                5 => context.checkpoint.preparation_recovery = None,
                6 => {
                    context.checkpoint.observations[0].case = Some(DesktopEvidenceCase::GuildJoined)
                }
                7 => context.checkpoint.callback.state.profile.motto = "Changed".to_owned(),
                _ => context.checkpoint.last_public_cells = Some(vec![]),
            }
            assert!(
                reconcile_operator_join(
                    &mut context,
                    &AtomicBool::new(false),
                    Instant::now() + Duration::from_secs(60),
                )
                .is_err()
            );
            assert!(context.checkpoint.operator_accepted_join.is_none());
        }
        context.checkpoint = serde_json::from_slice(&original).unwrap();
        for condition in 0..4 {
            match condition {
                0 => transport.abnormal.set(true),
                1 => {
                    transport.abnormal.set(false);
                    transport.cells.borrow_mut()[3] = "2".to_owned();
                }
                2 => {
                    transport.cells.borrow_mut()[3] = "1".to_owned();
                    *transport.guild.borrow_mut() = "Other".to_owned();
                }
                _ => {
                    *transport.guild.borrow_mut() = "BEERguild".to_owned();
                    transport.page_delay_milliseconds.set(5);
                }
            }
            let deadline = Instant::now()
                + if condition == 3 {
                    Duration::from_millis(1)
                } else {
                    Duration::from_secs(60)
                };
            assert!(
                reconcile_operator_join(&mut context, &AtomicBool::new(false), deadline).is_err()
            );
            assert!(context.checkpoint.operator_accepted_join.is_none());
        }
        transport.page_delay_milliseconds.set(0);
        let pages = transport.pages.get();
        assert!(
            reconcile_operator_join(&mut context, &AtomicBool::new(false), Instant::now()).is_err()
        );
        assert_eq!(transport.pages.get(), pages);
        assert_eq!(transport.calls.get(), calls);
        context.save.profile.motto = "Changed source".to_owned();
        assert!(operator_join_intent(&context).is_err());
    }

    #[test]
    fn scoped_guild_cases_reconcile_case_only_differences() {
        let directory = TestDirectory::new();
        let mut options = options(&directory.0);
        options.guild_designation = "gUiLd A".to_owned();
        options.guild_change_designation = Some("gUiLd B".to_owned());
        let transport = fake(&options, false);
        let mut context = context(&options, &transport);
        immediate(&mut context, &AtomicBool::new(false)).unwrap();
        assert_eq!(transport.calls.get(), 7);
        assert!(complete(&context, DesktopEvidenceCase::GuildJoined));
        assert!(complete(&context, DesktopEvidenceCase::GuildChanged));
        assert!(evidence_records(&context, true).is_ok());
    }

    #[test]
    fn recovery_requires_explicit_exact_immediate_scope() {
        let directory = TestDirectory::new();
        let mut options = existing_options(&directory.0);
        options.preparation_reconciliation_seconds = Some(60);
        assert!(validate_scope(&options).is_err());
        options.confirm_preparation_reconciliation = true;
        assert!(validate_scope(&options).is_ok());
        options.preparation_reconciliation_seconds = Some(61);
        assert!(validate_scope(&options).is_err());
        options.preparation_reconciliation_seconds = Some(60);
        options.max_mutation_attempts = Some(7);
        assert!(validate_scope(&options).is_err());
        options.max_mutation_attempts = Some(8);
        options.initial_guild_leave = false;
        assert!(validate_scope(&options).is_err());
        options.initial_guild_leave = true;
        options.operations.pop();
        assert!(validate_scope(&options).is_err());
        options.stage = DesktopLiveStage::Progression;
        assert!(validate_scope(&options).is_err());
    }

    fn record_uncertain_preparation(context: &mut Context<'_>) {
        context.save.profile.guild = "Original Guild".to_owned();
        context.checkpoint.callback.state = DesktopCanonicalState::from(&context.save);
        let request = guild_request(
            &context.checkpoint.callback.state,
            context.contract.realm,
            "",
            context.save.private.passkey,
            &DesktopAccountAuthentication::new("", ""),
        )
        .unwrap();
        context.checkpoint.attempts.push(request_intent_sha256(
            "initial-guild-leave",
            &request.encoded_query(),
        ));
        persist(context).unwrap();
    }

    #[test]
    fn read_only_recovery_preserves_intent_and_resumes_only_seven_primary_cases() {
        let directory = TestDirectory::new();
        let mut options = existing_options(&directory.0);
        options.preparation_reconciliation_seconds = Some(60);
        options.confirm_preparation_reconciliation = true;
        let transport = fake(&options, false);
        let mut context = context(&options, &transport);
        record_uncertain_preparation(&mut context);
        let original_intent = context.checkpoint.attempts.clone();
        reconcile_preparation(&mut context, &AtomicBool::new(false)).unwrap();
        assert_eq!(transport.calls.get(), 0);
        assert_eq!(context.checkpoint.attempts, original_intent);
        assert!(context.checkpoint.observations.is_empty());
        assert!(!complete(&context, DesktopEvidenceCase::GuildLeft));
        assert!(evidence_records(&context, true).is_err());
        let private = fs::read(&context.checkpoint_path).unwrap();
        assert!(
            !String::from_utf8(private.clone())
                .unwrap()
                .contains("fingerprintSha256")
        );
        context.checkpoint = serde_json::from_slice(&private).unwrap();
        assert!(checkpoint_can_resume(&context.checkpoint));
        immediate(&mut context, &AtomicBool::new(false)).unwrap();
        assert_eq!(transport.calls.get(), 7);
        assert_eq!(context.checkpoint.attempts.len(), 8);
        assert_eq!(context.checkpoint.observations.len(), 7);
        assert_eq!(context.checkpoint.attempts[0], original_intent[0]);
        assert_eq!(evidence_records(&context, true).unwrap().len(), 3);
        immediate(&mut context, &AtomicBool::new(false)).unwrap();
        assert_eq!(transport.calls.get(), 7);
    }

    #[test]
    fn recovery_refuses_unknown_intents_changed_state_progress_and_expiration() {
        let directory = TestDirectory::new();
        let mut options = existing_options(&directory.0);
        options.preparation_reconciliation_seconds = Some(60);
        options.confirm_preparation_reconciliation = true;
        let transport = fake(&options, false);
        let mut context = context(&options, &transport);
        record_uncertain_preparation(&mut context);
        let original = serde_json::to_vec(&context.checkpoint).unwrap();
        assert!(
            validate_checkpoint_scope(&context.checkpoint, "changed-source-scope", &options)
                .is_err()
        );
        for invalid in 0..7 {
            context.checkpoint = serde_json::from_slice(&original).unwrap();
            match invalid {
                0 => context.checkpoint.attempts[0] = "unknown-primary-intent".to_owned(),
                1 => context.checkpoint.attempts.push("second".to_owned()),
                2 => context.checkpoint.active_milliseconds = 1,
                3 => context.checkpoint.pending_callback = true,
                4 => context.checkpoint.callback.state.profile.motto = "Changed".to_owned(),
                5 => context.checkpoint.last_public_cells = Some(vec![]),
                _ => context.checkpoint.observations.push(StageObservation {
                    intent_sha256: context.checkpoint.attempts[0].clone(),
                    case: None,
                    fingerprint_sha256: "0".repeat(64),
                    observed_on: current_utc_date(),
                }),
            }
            assert!(reconcile_preparation(&mut context, &AtomicBool::new(false)).is_err());
            assert!(context.checkpoint.preparation_recovery.is_none());
        }
        context.checkpoint = serde_json::from_slice(&original).unwrap();
        let pages = transport.pages.get();
        assert!(
            reconcile_preparation_until(&mut context, &AtomicBool::new(false), Instant::now(),)
                .is_err()
        );
        assert_eq!(transport.pages.get(), pages);
        assert_eq!(transport.calls.get(), 0);
        assert!(context.checkpoint.preparation_recovery.is_none());
        transport.cells.borrow_mut()[3] = "2".to_owned();
        assert!(reconcile_preparation(&mut context, &AtomicBool::new(false)).is_err());
        transport.cells.borrow_mut()[3] = "1".to_owned();
        *transport.guild.borrow_mut() = "Original Guild".to_owned();
        assert!(reconcile_preparation(&mut context, &AtomicBool::new(false)).is_err());
        assert_eq!(transport.calls.get(), 0);
    }

    #[test]
    fn preparation_is_counted_but_cannot_replace_final_leave_proof() {
        let directory = TestDirectory::new();
        let options = existing_options(&directory.0);
        let transport = fake(&options, false);
        *transport.guild.borrow_mut() = "Original Guild".to_owned();
        let mut context = context(&options, &transport);
        context.checkpoint.callback.state.profile.guild = "Original Guild".to_owned();
        prepare_initial_guild_leave(&mut context, &AtomicBool::new(false)).unwrap();
        assert_eq!(transport.calls.get(), 1);
        assert_eq!(context.checkpoint.observations[0].case, None);
        assert!(!complete(&context, DesktopEvidenceCase::GuildLeft));
        assert!(evidence_records(&context, true).is_err());
        immediate(&mut context, &AtomicBool::new(false)).unwrap();
        assert_eq!(transport.calls.get(), 8);
        assert_eq!(context.checkpoint.callback.state.profile.guild, "");
        assert_eq!(evidence_records(&context, true).unwrap().len(), 3);
        immediate(&mut context, &AtomicBool::new(false)).unwrap();
        assert_eq!(transport.calls.get(), 8);
        context
            .checkpoint
            .observations
            .retain(|value| value.case != Some(DesktopEvidenceCase::GuildLeft));
        assert!(evidence_records(&context, true).is_err());
    }

    #[test]
    fn mismatched_baseline_and_uncertain_preparation_never_replay() {
        let directory = TestDirectory::new();
        let options = existing_options(&directory.0);
        let transport = fake(&options, true);
        *transport.guild.borrow_mut() = "Original Guild".to_owned();
        let mut context = context(&options, &transport);
        context.checkpoint.callback.state.profile.guild = "Different Guild".to_owned();
        assert!(immediate(&mut context, &AtomicBool::new(false)).is_err());
        assert_eq!(transport.calls.get(), 0);
        context.checkpoint.callback.state.profile.guild = "Original Guild".to_owned();
        transport.cells.borrow_mut()[3] = "2".to_owned();
        assert!(immediate(&mut context, &AtomicBool::new(false)).is_err());
        assert_eq!(transport.calls.get(), 0);
        transport.cells.borrow_mut()[3] = "1".to_owned();
        transport.pages.set(0);
        assert!(immediate(&mut context, &AtomicBool::new(false)).is_err());
        assert_eq!(transport.calls.get(), 1);
        assert!(!checkpoint_can_resume(&context.checkpoint));
        assert!(immediate(&mut context, &AtomicBool::new(false)).is_err());
        assert_eq!(transport.calls.get(), 1);
        assert!(evidence_records(&context, true).is_err());
    }
    #[test]
    fn immediate_cases_emit_independent_valid_records_only_after_cleanup() {
        let directory = TestDirectory::new();
        let options = options(&directory.0);
        let transport = fake(&options, false);
        let mut context = context(&options, &transport);
        immediate(&mut context, &AtomicBool::new(false)).unwrap();
        assert_eq!(transport.calls.get(), 7);
        assert!(checkpoint_can_resume(&context.checkpoint));
        assert_eq!(context.checkpoint.callback.state.profile.guild, "");
        assert!(evidence_records(&context, false).is_err());
        let records = evidence_records(&context, true).unwrap();
        assert_eq!(records.len(), 3);
        for &operation in &options.operations {
            let value = &records[&operation.label().replace(' ', "-")];
            let content = serde_json::to_string(value).unwrap();
            crate::fixtures::validate_fixture(Path::new("stage-operation-evidence.json"), &content)
                .unwrap();
            let validated = validate_desktop_evidence(
                &content,
                &DesktopEvidenceExpectation {
                    realm: "Pemptus",
                    saved_endpoint: context.contract.saved_endpoint,
                    verified_https_endpoint: context.contract.https_endpoint,
                    operation,
                    current_date: &current_utc_date(),
                    integrity_sha256: value["integritySha256"].as_str().unwrap(),
                },
            )
            .unwrap();
            assert_eq!(validated.eligibility().operations, [operation]);
        }
        let guild = &records["guild"];
        crate::desktop_evidence::with_synthetic_desktop_evidence(
            vec![crate::desktop_evidence::SyntheticDesktopEvidence {
                contract: context.contract,
                operation: DesktopOnlineOperation::Guild,
                adaptations: vec![],
                content: serde_json::to_string(guild).unwrap(),
                integrity: guild["integritySha256"].as_str().unwrap().to_owned(),
            }],
            || {
                let values = DesktopGuildFingerprintValues {
                    character_name: &context.name,
                    account: "",
                    password: "",
                    authorization: "",
                    passkey: "42424",
                    prior_guild: "Guild B",
                    submitted_guild: "__gyrognome-invalid__",
                };
                let fingerprints = crate::desktop_evidence::production_desktop_evidence(
                    context.contract,
                    DesktopOnlineOperation::Guild,
                )
                .unwrap()
                .guild_fingerprints()
                .unwrap();
                assert_eq!(fingerprints.join, values.fingerprint(b"joined"));
                assert_eq!(fingerprints.change, Some(values.fingerprint(b"changed")));
                assert_eq!(fingerprints.rejected, values.fingerprint(b"rejected"));
                assert_eq!(fingerprints.leave, values.fingerprint(b"left"));
                let rules = crate::desktop_profile::DesktopGuildResponseRules::production(
                    "Pemptus",
                    crate::desktop_profile::DesktopGuildOperation::JoinOrChange,
                )
                .unwrap();
                for body in [b"joined".as_slice(), b"changed".as_slice()] {
                    assert_eq!(
                        rules.classify(200, body, &values),
                        crate::desktop_profile::DesktopGuildOutcome::Indeterminate
                    );
                }
                assert_eq!(
                    rules.classify(200, b"rejected", &values),
                    crate::desktop_profile::DesktopGuildOutcome::Indeterminate
                );
                let leave = crate::desktop_profile::DesktopGuildResponseRules::production(
                    "Pemptus",
                    crate::desktop_profile::DesktopGuildOperation::Leave,
                )
                .unwrap();
                assert_eq!(
                    leave.classify(200, b"left", &values),
                    crate::desktop_profile::DesktopGuildOutcome::Indeterminate
                );
            },
        );
        assert!(
            crate::desktop_evidence::production_desktop_evidence(
                crate::desktop_contract::PEMPTUS,
                DesktopOnlineOperation::Guild,
            )
            .is_err()
        );
        for observation in &mut context.checkpoint.observations {
            observation.observed_on = "2020-09-30".to_owned();
        }
        assert!(evidence_records(&context, true).is_err());
    }

    #[test]
    fn unchanged_http_success_cannot_complete_or_replay_manual_intent() {
        let directory = TestDirectory::new();
        let mut options = options(&directory.0);
        options.operations = vec![DesktopOnlineOperation::ManualBrag];
        let transport = fake(&options, true);
        let mut context = context(&options, &transport);
        assert!(immediate(&mut context, &AtomicBool::new(false)).is_err());
        assert_eq!(transport.calls.get(), 1);
        assert!(context.checkpoint.observations.is_empty());
        assert!(!checkpoint_can_resume(&context.checkpoint));
        assert!(evidence_records(&context, true).is_err());
        assert!(immediate(&mut context, &AtomicBool::new(false)).is_err());
        assert_eq!(transport.calls.get(), 1);
    }

    #[test]
    fn interruption_bound_expiration_and_pending_callbacks_are_non_enabling() {
        let directory = TestDirectory::new();
        let options = options(&directory.0);
        let transport = fake(&options, false);
        let mut context = context(&options, &transport);
        assert!(matches!(
            immediate(&mut context, &AtomicBool::new(true)),
            Err(DesktopLiveExperimentError::Interrupted)
        ));
        assert_eq!(transport.calls.get(), 0);
        context.checkpoint.active_milliseconds = options.max_active_seconds * 1_000;
        assert!(matches!(
            progression(&mut context, &AtomicBool::new(false)),
            Err(DesktopLiveExperimentError::BoundExpired)
        ));
        assert!(evidence_records(&context, true).is_err());
        context.checkpoint.pending_callback = true;
        assert!(!checkpoint_can_resume(&context.checkpoint));
        let observation = context
            .checkpoint
            .callback
            .apply_progression_callback(30_000, &mut SourceDerivedDesktopHooks::traced())
            .unwrap();
        assert_eq!(observation.credited_milliseconds, 100);
        assert!(!observation.completion_dispatched);
    }

    #[test]
    fn cleanup_is_exact_and_refuses_symlinks_without_removing_unrelated_files() {
        let directory = TestDirectory::new();
        let checkpoint = directory.0.join(CHECKPOINT);
        let save = directory.0.join("disposable.pq");
        let backup = directory.0.join("disposable.bak");
        let unrelated = directory.0.join("unrelated.json");
        for path in [&checkpoint, &save, &unrelated] {
            fs::write(path, b"synthetic").unwrap();
        }
        std::os::unix::fs::symlink(&unrelated, &backup).unwrap();
        assert!(cleanup_files(&checkpoint, &save, Some(&backup)).is_err());
        assert!(checkpoint.exists() && save.exists() && unrelated.exists());
        fs::remove_file(&backup).unwrap();
        fs::write(&backup, b"synthetic").unwrap();
        cleanup_files(&checkpoint, &save, Some(&backup)).unwrap();
        assert!(!checkpoint.exists() && !save.exists() && !backup.exists());
        assert!(unrelated.exists());
    }

    #[test]
    fn private_copy_cleanup_preserves_original_save_and_backup() {
        let directory = TestDirectory::new();
        let originals = directory.0.join("originals");
        let private = directory.0.join("private");
        fs::create_dir(&originals).unwrap();
        fs::create_dir(&private).unwrap();
        let original_save = originals.join("original.pq");
        let original_backup = originals.join("original.bak");
        fs::write(&original_save, b"original save").unwrap();
        fs::write(&original_backup, b"original backup").unwrap();
        let save = private.join("disposable.pq");
        let backup = private.join("disposable.bak");
        let checkpoint = private.join(CHECKPOINT);
        fs::copy(&original_save, &save).unwrap();
        fs::copy(&original_backup, &backup).unwrap();
        fs::write(&checkpoint, b"synthetic ledger").unwrap();
        cleanup_files(&checkpoint, &save, Some(&backup)).unwrap();
        assert_eq!(fs::read(&original_save).unwrap(), b"original save");
        assert_eq!(fs::read(&original_backup).unwrap(), b"original backup");
        assert!(!save.exists() && !backup.exists() && !checkpoint.exists());
    }

    #[test]
    fn failed_source_cleanup_retains_the_intent_ledger() {
        for fail_backup in [false, true] {
            let directory = TestDirectory::new();
            let checkpoint = directory.0.join(CHECKPOINT);
            let save = directory.0.join("disposable.pq");
            let backup = directory.0.join("disposable.bak");
            for path in [&checkpoint, &save, &backup] {
                fs::write(path, b"synthetic").unwrap();
            }
            let failed_path = if fail_backup { &backup } else { &save };
            assert!(
                cleanup_files_with(&checkpoint, &save, Some(&backup), |path| {
                    if path == failed_path {
                        Err(std::io::Error::from(std::io::ErrorKind::PermissionDenied))
                    } else {
                        fs::remove_file(path)
                    }
                })
                .is_err()
            );
            assert!(checkpoint.exists());
            assert!(backup.exists());
            assert_eq!(save.exists(), !fail_backup);
        }
    }
}
