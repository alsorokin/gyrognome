use std::{
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use ring::digest::{SHA256, digest};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::Url;

use crate::{
    compatibility::{CompatibilityProfile, DesktopAdaptation, DesktopLayout, SourceFormat},
    desktop_eligibility::{
        DesktopCredentialMode, DesktopEligibilityEvidence, DesktopOnlineOperation,
    },
    desktop_fingerprint::DESKTOP_RESPONSE_FINGERPRINT_VERSION,
    desktop_rules::{CONFIG_DFM_SHA256, MAIN_PAS_SHA256, SOURCE_COMMIT, SOURCE_TAG},
    fixtures::validate_fixture,
};

pub const DESKTOP_EVIDENCE_FORMAT: &str = "gyrognome-desktop-live-evidence/v2";
pub const DESKTOP_OPERATION_EVIDENCE_FORMAT: &str = "gyrognome-desktop-operation-evidence/v3";
pub const DESKTOP_ONLINE_IMPLEMENTATION_ID: &str = "desktop-online-contract/v1";
pub const PRODUCTION_DESKTOP_EVIDENCE_INTEGRITY: &str =
    "edd7f9aa16b5be4f4a695a12d417e3505e55aeb5d5d68b180b8d41f16a3860f1";
const PRODUCTION_DESKTOP_EVIDENCE: &str =
    include_str!("../tests/fixtures/desktop-online-evidence.json");
const PEMPTUS_MANUAL_EVIDENCE: &str =
    include_str!("../tests/fixtures/pemptus-manual-operation-evidence.json");
const PEMPTUS_MANUAL_EVIDENCE_INTEGRITY: &str =
    "b5364c417826345ce6fcc4735af1489f2a283ebdcb00237731bbe6d4b2b34d46";
const PEMPTUS_MOTTO_EVIDENCE: &str =
    include_str!("../tests/fixtures/pemptus-motto-operation-evidence.json");
const PEMPTUS_MOTTO_EVIDENCE_INTEGRITY: &str =
    "10ca6efa9d2411dcb91c67d89281e428d743b00574694faa38f759011db8efd6";

#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
pub enum DesktopEvidenceError {
    #[error("desktop online evidence is unavailable")]
    Unavailable,
    #[error("desktop online evidence is malformed")]
    Malformed,
    #[error("desktop online evidence contains sensitive data")]
    Sensitive,
    #[error("desktop online evidence integrity check failed")]
    Integrity,
    #[error("desktop online evidence is stale")]
    Stale,
    #[error("desktop online evidence identity does not match")]
    IdentityMismatch,
    #[error("desktop online evidence profile does not match")]
    ProfileMismatch,
    #[error("desktop online evidence realm does not match")]
    RealmMismatch,
    #[error("desktop online evidence endpoint does not match")]
    EndpointMismatch,
    #[error("desktop online evidence operation does not match")]
    OperationMismatch,
    #[error("desktop online evidence result is not conclusively passing")]
    Inconclusive,
}

pub struct DesktopEvidenceExpectation<'a> {
    pub realm: &'a str,
    pub saved_endpoint: &'a str,
    pub verified_https_endpoint: &'a str,
    pub operation: DesktopOnlineOperation,
    pub current_date: &'a str,
    pub integrity_sha256: &'a str,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct ValidatedDesktopEvidence {
    payload: DesktopEvidencePayload,
}

impl ValidatedDesktopEvidence {
    #[allow(dead_code)]
    pub(crate) fn eligibility(&self) -> DesktopEligibilityEvidence<'_> {
        DesktopEligibilityEvidence {
            profile: self.payload.target_profile,
            source_format: self.payload.import_path.source_format,
            layout: self.payload.import_path.layout,
            adaptations: &self.payload.import_path.adaptations,
            realm: &self.payload.realm,
            saved_endpoint: &self.payload.saved_endpoint,
            credential_mode: self.payload.credential_mode,
            operations: &self.payload.operations,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct DesktopEvidenceEnvelope {
    format: String,
    payload: DesktopEvidencePayload,
    integrity_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct DesktopEvidencePayload {
    status: String,
    target_profile: CompatibilityProfile,
    source_identity: DesktopEvidenceSourceIdentity,
    implementation_identity: String,
    import_path: DesktopEvidenceImportPath,
    realm: String,
    saved_endpoint: String,
    verified_https_endpoint: String,
    credential_mode: DesktopCredentialMode,
    operations: Vec<DesktopOnlineOperation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    guild_responses: Option<DesktopGuildResponseEvidence>,
    classification: String,
    cleanup_complete: bool,
    observed_on: String,
    valid_through: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    encoding: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    accepted_cases: Option<Vec<DesktopEvidenceCase>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DesktopEvidenceCase {
    ManualConsumed,
    MottoSet,
    MottoCleared,
    GuildJoined,
    GuildChanged,
    GuildRejected,
    GuildLeft,
    LevelAccepted,
    ActAccepted,
}

impl DesktopEvidenceCase {
    pub fn required(operation: DesktopOnlineOperation) -> &'static [Self] {
        match operation {
            DesktopOnlineOperation::AutomaticLevel => &[Self::LevelAccepted],
            DesktopOnlineOperation::AutomaticAct => &[Self::ActAccepted],
            DesktopOnlineOperation::ManualBrag => &[Self::ManualConsumed],
            DesktopOnlineOperation::Motto => &[Self::MottoSet, Self::MottoCleared],
            DesktopOnlineOperation::Guild => &[
                Self::GuildJoined,
                Self::GuildChanged,
                Self::GuildRejected,
                Self::GuildLeft,
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct DesktopEvidenceSourceIdentity {
    tag: String,
    commit: String,
    main_pas_sha256: String,
    config_dfm_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct DesktopEvidenceImportPath {
    source_format: SourceFormat,
    layout: DesktopLayout,
    adaptations: Vec<DesktopAdaptation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct DesktopGuildResponseEvidence {
    normalization: String,
    join_accepted_fingerprint_sha256: String,
    rejected_fingerprint_sha256: String,
    leave_accepted_fingerprint_sha256: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    change_accepted_fingerprint_sha256: Option<String>,
}

pub fn validate_desktop_evidence(
    content: &str,
    expected: &DesktopEvidenceExpectation<'_>,
) -> Result<ValidatedDesktopEvidence, DesktopEvidenceError> {
    if validate_fixture(Path::new("desktop-online-evidence.json"), content).is_err() {
        return Err(DesktopEvidenceError::Sensitive);
    }
    let envelope: DesktopEvidenceEnvelope =
        serde_json::from_str(content).map_err(|_| DesktopEvidenceError::Malformed)?;
    let operation_scoped = envelope.format == DESKTOP_OPERATION_EVIDENCE_FORMAT;
    if envelope.format != DESKTOP_EVIDENCE_FORMAT && !operation_scoped {
        return Err(DesktopEvidenceError::Malformed);
    }
    if !operation_scoped
        && (envelope.payload.encoding.is_some()
            || envelope.payload.accepted_cases.is_some()
            || envelope
                .payload
                .guild_responses
                .as_ref()
                .is_some_and(|guild| guild.change_accepted_fingerprint_sha256.is_some()))
    {
        return Err(DesktopEvidenceError::Malformed);
    }
    let computed = payload_sha256(&envelope.payload)?;
    if !valid_sha256(&envelope.integrity_sha256)
        || envelope.integrity_sha256 != computed
        || envelope.integrity_sha256 != expected.integrity_sha256
    {
        return Err(DesktopEvidenceError::Integrity);
    }
    if !valid_date(&envelope.payload.observed_on)
        || !valid_date(&envelope.payload.valid_through)
        || !valid_date(expected.current_date)
    {
        return Err(DesktopEvidenceError::Malformed);
    }
    if envelope.payload.observed_on > envelope.payload.valid_through {
        return Err(DesktopEvidenceError::Malformed);
    }
    if envelope.payload.valid_through.as_str() < expected.current_date
        || (operation_scoped && envelope.payload.observed_on.as_str() > expected.current_date)
    {
        return Err(DesktopEvidenceError::Stale);
    }
    if envelope.payload.source_identity.tag != SOURCE_TAG
        || envelope.payload.source_identity.commit != SOURCE_COMMIT
        || envelope.payload.source_identity.main_pas_sha256 != MAIN_PAS_SHA256
        || envelope.payload.source_identity.config_dfm_sha256 != CONFIG_DFM_SHA256
        || (!operation_scoped
            && envelope.payload.implementation_identity != DESKTOP_ONLINE_IMPLEMENTATION_ID)
    {
        return Err(DesktopEvidenceError::IdentityMismatch);
    }
    if envelope.payload.target_profile != CompatibilityProfile::Desktop644 {
        return Err(DesktopEvidenceError::ProfileMismatch);
    }
    if envelope.payload.import_path.source_format != SourceFormat::DesktopDelphiComponentStream
        || envelope.payload.import_path.layout != DesktopLayout::SupportedComponentStream
    {
        return Err(DesktopEvidenceError::IdentityMismatch);
    }
    if envelope.payload.realm != expected.realm {
        return Err(DesktopEvidenceError::RealmMismatch);
    }
    if envelope.payload.saved_endpoint != expected.saved_endpoint {
        return Err(DesktopEvidenceError::EndpointMismatch);
    }
    if envelope.payload.verified_https_endpoint != expected.verified_https_endpoint
        || !valid_verified_endpoint(&envelope.payload.verified_https_endpoint)
    {
        return Err(DesktopEvidenceError::EndpointMismatch);
    }
    if !envelope.payload.operations.contains(&expected.operation) {
        return Err(DesktopEvidenceError::OperationMismatch);
    }
    if envelope.payload.operations.is_empty() || envelope.payload.operations.len() > 5 {
        return Err(DesktopEvidenceError::Malformed);
    }
    if operation_scoped {
        let contract = crate::desktop_contract::realm_contract(expected.realm)
            .map_err(|_| DesktopEvidenceError::RealmMismatch)?;
        if contract.saved_endpoint != expected.saved_endpoint
            || contract.https_endpoint != expected.verified_https_endpoint
        {
            return Err(DesktopEvidenceError::EndpointMismatch);
        }
        if envelope.payload.credential_mode != contract.credential_mode
            || envelope.payload.implementation_identity != contract.implementation_identity
        {
            return Err(DesktopEvidenceError::IdentityMismatch);
        }
        if envelope.payload.operations != [expected.operation]
            || envelope.payload.encoding.as_deref() != Some("ascii")
            || envelope.payload.accepted_cases.as_deref()
                != Some(DesktopEvidenceCase::required(expected.operation))
            || !envelope.payload.import_path.adaptations.is_empty()
        {
            return Err(DesktopEvidenceError::Inconclusive);
        }
    }
    if !operation_scoped || expected.operation == DesktopOnlineOperation::Guild {
        let guild = envelope
            .payload
            .guild_responses
            .as_ref()
            .ok_or(DesktopEvidenceError::Malformed)?;
        if guild.normalization != DESKTOP_RESPONSE_FINGERPRINT_VERSION
            || ![
                &guild.join_accepted_fingerprint_sha256,
                &guild.rejected_fingerprint_sha256,
                &guild.leave_accepted_fingerprint_sha256,
            ]
            .into_iter()
            .all(|value| valid_sha256(value))
            || (operation_scoped
                && !guild
                    .change_accepted_fingerprint_sha256
                    .as_ref()
                    .is_some_and(|value| valid_sha256(value)))
            || (operation_scoped
                && [
                    Some(&guild.join_accepted_fingerprint_sha256),
                    guild.change_accepted_fingerprint_sha256.as_ref(),
                    Some(&guild.leave_accepted_fingerprint_sha256),
                ]
                .into_iter()
                .flatten()
                .any(|value| value == &guild.rejected_fingerprint_sha256))
        {
            return Err(DesktopEvidenceError::Malformed);
        }
    } else if envelope.payload.guild_responses.is_some() {
        return Err(DesktopEvidenceError::Malformed);
    }
    if envelope.payload.status != "passing"
        || envelope.payload.classification != "normal"
        || !envelope.payload.cleanup_complete
    {
        return Err(DesktopEvidenceError::Inconclusive);
    }
    Ok(ValidatedDesktopEvidence {
        payload: envelope.payload,
    })
}

pub fn validate_production_desktop_evidence(
    expected: &DesktopEvidenceExpectation<'_>,
) -> Result<ValidatedDesktopEvidence, DesktopEvidenceError> {
    validate_desktop_evidence(PRODUCTION_DESKTOP_EVIDENCE, expected)
}

pub(crate) struct DesktopEvidenceRecord<'a> {
    pub contract: crate::desktop_contract::DesktopRealmContract,
    pub operation: DesktopOnlineOperation,
    pub content: &'a str,
    pub integrity: &'a str,
}

pub(crate) fn select_desktop_evidence(
    records: &[DesktopEvidenceRecord<'_>],
    contract: crate::desktop_contract::DesktopRealmContract,
    operation: DesktopOnlineOperation,
    current_date: &str,
) -> Result<ValidatedDesktopEvidence, DesktopEvidenceError> {
    let mut matching = records
        .iter()
        .filter(|record| record.contract == contract && record.operation == operation);
    let record = matching.next().ok_or(DesktopEvidenceError::Unavailable)?;
    if matching.next().is_some() {
        return Err(DesktopEvidenceError::Malformed);
    }
    let validated = validate_desktop_evidence(
        record.content,
        &DesktopEvidenceExpectation {
            realm: contract.realm,
            saved_endpoint: contract.saved_endpoint,
            verified_https_endpoint: contract.https_endpoint,
            operation,
            current_date,
            integrity_sha256: record.integrity,
        },
    )?;
    if validated.payload.credential_mode != contract.credential_mode {
        return Err(DesktopEvidenceError::IdentityMismatch);
    }
    Ok(validated)
}

pub(crate) fn production_desktop_evidence(
    contract: crate::desktop_contract::DesktopRealmContract,
    operation: DesktopOnlineOperation,
) -> Result<ValidatedDesktopEvidence, DesktopEvidenceError> {
    #[cfg(test)]
    if let Some(result) = TEST_EVIDENCE.with(|records| {
        records.borrow().as_ref().map(|records| {
            let borrowed: Vec<_> = records
                .iter()
                .map(|record| DesktopEvidenceRecord {
                    contract: record.contract,
                    operation: record.operation,
                    content: &record.content,
                    integrity: &record.integrity,
                })
                .collect();
            select_desktop_evidence(&borrowed, contract, operation, &current_utc_date())
        })
    }) {
        return result;
    }
    select_desktop_evidence(
        &production_records(),
        contract,
        operation,
        &current_utc_date(),
    )
}

fn production_records() -> Vec<DesktopEvidenceRecord<'static>> {
    let mut records: Vec<_> = [
        DesktopOnlineOperation::AutomaticLevel,
        DesktopOnlineOperation::AutomaticAct,
        DesktopOnlineOperation::ManualBrag,
        DesktopOnlineOperation::Motto,
        DesktopOnlineOperation::Guild,
    ]
    .map(|operation| DesktopEvidenceRecord {
        contract: crate::desktop_contract::SPOLTOG,
        operation,
        content: PRODUCTION_DESKTOP_EVIDENCE,
        integrity: PRODUCTION_DESKTOP_EVIDENCE_INTEGRITY,
    })
    .into();
    records.extend([
        DesktopEvidenceRecord {
            contract: crate::desktop_contract::PEMPTUS,
            operation: DesktopOnlineOperation::ManualBrag,
            content: PEMPTUS_MANUAL_EVIDENCE,
            integrity: PEMPTUS_MANUAL_EVIDENCE_INTEGRITY,
        },
        DesktopEvidenceRecord {
            contract: crate::desktop_contract::PEMPTUS,
            operation: DesktopOnlineOperation::Motto,
            content: PEMPTUS_MOTTO_EVIDENCE,
            integrity: PEMPTUS_MOTTO_EVIDENCE_INTEGRITY,
        },
    ]);
    records
}

#[cfg(test)]
#[derive(Clone)]
pub(crate) struct SyntheticDesktopEvidence {
    pub contract: crate::desktop_contract::DesktopRealmContract,
    pub operation: DesktopOnlineOperation,
    pub content: String,
    pub integrity: String,
}

#[cfg(test)]
thread_local! {
    static TEST_EVIDENCE: std::cell::RefCell<Option<Vec<SyntheticDesktopEvidence>>> =
        const { std::cell::RefCell::new(None) };
}

#[cfg(test)]
pub(crate) fn with_synthetic_desktop_evidence<R>(
    records: Vec<SyntheticDesktopEvidence>,
    action: impl FnOnce() -> R,
) -> R {
    struct Restore(Option<Vec<SyntheticDesktopEvidence>>);
    impl Drop for Restore {
        fn drop(&mut self) {
            TEST_EVIDENCE.with(|records| records.replace(self.0.take()));
        }
    }
    let _restore = Restore(TEST_EVIDENCE.with(|current| current.replace(Some(records))));
    action()
}

#[cfg(test)]
pub(crate) fn synthetic_pemptus_evidence(
    operation: DesktopOnlineOperation,
    fingerprints: Option<DesktopGuildFingerprints>,
) -> SyntheticDesktopEvidence {
    let contract = crate::desktop_contract::PEMPTUS;
    let payload = DesktopEvidencePayload {
        status: "passing".to_owned(),
        target_profile: CompatibilityProfile::Desktop644,
        source_identity: DesktopEvidenceSourceIdentity {
            tag: SOURCE_TAG.to_owned(),
            commit: SOURCE_COMMIT.to_owned(),
            main_pas_sha256: MAIN_PAS_SHA256.to_owned(),
            config_dfm_sha256: CONFIG_DFM_SHA256.to_owned(),
        },
        implementation_identity: contract.implementation_identity.to_owned(),
        import_path: DesktopEvidenceImportPath {
            source_format: SourceFormat::DesktopDelphiComponentStream,
            layout: DesktopLayout::SupportedComponentStream,
            adaptations: vec![],
        },
        realm: contract.realm.to_owned(),
        saved_endpoint: contract.saved_endpoint.to_owned(),
        verified_https_endpoint: contract.https_endpoint.to_owned(),
        credential_mode: contract.credential_mode,
        operations: vec![operation],
        guild_responses: fingerprints.map(|guild| DesktopGuildResponseEvidence {
            normalization: guild.normalization,
            join_accepted_fingerprint_sha256: guild.join,
            change_accepted_fingerprint_sha256: guild.change,
            rejected_fingerprint_sha256: guild.rejected,
            leave_accepted_fingerprint_sha256: guild.leave,
        }),
        classification: "normal".to_owned(),
        cleanup_complete: true,
        observed_on: current_utc_date(),
        valid_through: "2027-09-27".to_owned(),
        encoding: Some("ascii".to_owned()),
        accepted_cases: Some(DesktopEvidenceCase::required(operation).to_vec()),
    };
    let integrity = payload_sha256(&payload).unwrap();
    let content = serde_json::to_string(&DesktopEvidenceEnvelope {
        format: DESKTOP_OPERATION_EVIDENCE_FORMAT.to_owned(),
        payload,
        integrity_sha256: integrity.clone(),
    })
    .unwrap();
    SyntheticDesktopEvidence {
        contract,
        operation,
        content,
        integrity,
    }
}

pub(crate) struct DesktopGuildFingerprints {
    pub normalization: String,
    pub join: String,
    pub change: Option<String>,
    pub rejected: String,
    pub leave: String,
}

#[cfg(feature = "desktop-live-conformance")]
pub(crate) struct DesktopOperationEvidenceObservation {
    pub contract: crate::desktop_contract::DesktopRealmContract,
    pub operation: DesktopOnlineOperation,
    pub accepted_cases: Vec<DesktopEvidenceCase>,
    pub guild_fingerprints: Option<DesktopGuildFingerprints>,
    pub observed_on: String,
    pub valid_through: String,
    pub cleanup_complete: bool,
    pub normal_classification: bool,
}

#[cfg(feature = "desktop-live-conformance")]
pub(crate) fn encode_operation_evidence(
    observation: DesktopOperationEvidenceObservation,
) -> Result<String, DesktopEvidenceError> {
    let contract = observation.contract;
    let payload = DesktopEvidencePayload {
        status: "passing".to_owned(),
        target_profile: CompatibilityProfile::Desktop644,
        source_identity: DesktopEvidenceSourceIdentity {
            tag: SOURCE_TAG.to_owned(),
            commit: SOURCE_COMMIT.to_owned(),
            main_pas_sha256: MAIN_PAS_SHA256.to_owned(),
            config_dfm_sha256: CONFIG_DFM_SHA256.to_owned(),
        },
        implementation_identity: contract.implementation_identity.to_owned(),
        import_path: DesktopEvidenceImportPath {
            source_format: SourceFormat::DesktopDelphiComponentStream,
            layout: DesktopLayout::SupportedComponentStream,
            adaptations: vec![],
        },
        realm: contract.realm.to_owned(),
        saved_endpoint: contract.saved_endpoint.to_owned(),
        verified_https_endpoint: contract.https_endpoint.to_owned(),
        credential_mode: contract.credential_mode,
        operations: vec![observation.operation],
        guild_responses: observation
            .guild_fingerprints
            .map(|guild| DesktopGuildResponseEvidence {
                normalization: guild.normalization,
                join_accepted_fingerprint_sha256: guild.join,
                change_accepted_fingerprint_sha256: guild.change,
                rejected_fingerprint_sha256: guild.rejected,
                leave_accepted_fingerprint_sha256: guild.leave,
            }),
        classification: if observation.normal_classification {
            "normal"
        } else {
            "pending"
        }
        .to_owned(),
        cleanup_complete: observation.cleanup_complete,
        observed_on: observation.observed_on,
        valid_through: observation.valid_through,
        encoding: Some("ascii".to_owned()),
        accepted_cases: Some(observation.accepted_cases),
    };
    let integrity = payload_sha256(&payload)?;
    let envelope = DesktopEvidenceEnvelope {
        format: DESKTOP_OPERATION_EVIDENCE_FORMAT.to_owned(),
        payload,
        integrity_sha256: integrity,
    };
    let content =
        serde_json::to_string_pretty(&envelope).map_err(|_| DesktopEvidenceError::Malformed)?;
    validate_desktop_evidence(
        &content,
        &DesktopEvidenceExpectation {
            realm: contract.realm,
            saved_endpoint: contract.saved_endpoint,
            verified_https_endpoint: contract.https_endpoint,
            operation: observation.operation,
            current_date: &current_utc_date(),
            integrity_sha256: &envelope.integrity_sha256,
        },
    )?;
    Ok(content)
}

impl ValidatedDesktopEvidence {
    pub(crate) fn guild_fingerprints(&self) -> Option<DesktopGuildFingerprints> {
        if !self
            .payload
            .operations
            .contains(&DesktopOnlineOperation::Guild)
        {
            return None;
        }
        self.payload
            .guild_responses
            .as_ref()
            .map(|guild| DesktopGuildFingerprints {
                normalization: guild.normalization.clone(),
                join: guild.join_accepted_fingerprint_sha256.clone(),
                change: guild.change_accepted_fingerprint_sha256.clone(),
                rejected: guild.rejected_fingerprint_sha256.clone(),
                leave: guild.leave_accepted_fingerprint_sha256.clone(),
            })
    }
}

fn payload_sha256(payload: &DesktopEvidencePayload) -> Result<String, DesktopEvidenceError> {
    let bytes = serde_json::to_vec(payload).map_err(|_| DesktopEvidenceError::Malformed)?;
    Ok(digest(&SHA256, &bytes)
        .as_ref()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn valid_date(value: &str) -> bool {
    let shaped = value.len() == 10
        && value.as_bytes()[4] == b'-'
        && value.as_bytes()[7] == b'-'
        && value
            .bytes()
            .enumerate()
            .all(|(index, byte)| index == 4 || index == 7 || byte.is_ascii_digit());
    if !shaped {
        return false;
    }
    let (Ok(year), Ok(month), Ok(day)) = (
        value[..4].parse::<u32>(),
        value[5..7].parse::<u32>(),
        value[8..].parse::<u32>(),
    ) else {
        return false;
    };
    let days = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400)) => {
            29
        }
        2 => 28,
        _ => return false,
    };
    year > 0 && (1..=days).contains(&day)
}

fn valid_verified_endpoint(value: &str) -> bool {
    Url::parse(value).is_ok_and(|endpoint| {
        endpoint.scheme() == "https"
            && endpoint.host_str().is_some()
            && endpoint.port().is_none()
            && endpoint.username().is_empty()
            && endpoint.password().is_none()
            && endpoint.query().is_none()
            && endpoint.fragment().is_none()
    })
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
    use serde_json::{Value, json};

    use super::*;
    use crate::desktop_eligibility::{
        DesktopEligibilityDecision, DesktopEligibilityInput, evaluate_desktop_eligibility,
    };

    fn payload() -> DesktopEvidencePayload {
        DesktopEvidencePayload {
            status: "passing".to_owned(),
            target_profile: CompatibilityProfile::Desktop644,
            source_identity: DesktopEvidenceSourceIdentity {
                tag: SOURCE_TAG.to_owned(),
                commit: SOURCE_COMMIT.to_owned(),
                main_pas_sha256: MAIN_PAS_SHA256.to_owned(),
                config_dfm_sha256: CONFIG_DFM_SHA256.to_owned(),
            },
            implementation_identity: DESKTOP_ONLINE_IMPLEMENTATION_ID.to_owned(),
            import_path: DesktopEvidenceImportPath {
                source_format: SourceFormat::DesktopDelphiComponentStream,
                layout: DesktopLayout::SupportedComponentStream,
                adaptations: vec![DesktopAdaptation::LoadSpellingPatch],
            },
            realm: "Synthetic Realm".to_owned(),
            saved_endpoint: "http://synthetic.invalid/report?".to_owned(),
            verified_https_endpoint: "https://synthetic.invalid/report".to_owned(),
            credential_mode: DesktopCredentialMode::AccountPassword,
            operations: vec![DesktopOnlineOperation::ManualBrag],
            guild_responses: Some(DesktopGuildResponseEvidence {
                normalization: DESKTOP_RESPONSE_FINGERPRINT_VERSION.to_owned(),
                join_accepted_fingerprint_sha256: "1".repeat(64),
                rejected_fingerprint_sha256: "2".repeat(64),
                leave_accepted_fingerprint_sha256: "3".repeat(64),
                change_accepted_fingerprint_sha256: None,
            }),
            classification: "normal".to_owned(),
            cleanup_complete: true,
            observed_on: "2026-09-20".to_owned(),
            valid_through: "2026-10-20".to_owned(),
            encoding: None,
            accepted_cases: None,
        }
    }

    fn evidence_value() -> (Value, String) {
        let payload = payload();
        let integrity = payload_sha256(&payload).unwrap();
        (
            json!({
                "format": DESKTOP_EVIDENCE_FORMAT,
                "payload": payload,
                "integritySha256": integrity,
            }),
            integrity,
        )
    }

    fn expectation<'a>(integrity: &'a str) -> DesktopEvidenceExpectation<'a> {
        DesktopEvidenceExpectation {
            realm: "Synthetic Realm",
            saved_endpoint: "http://synthetic.invalid/report?",
            verified_https_endpoint: "https://synthetic.invalid/report",
            operation: DesktopOnlineOperation::ManualBrag,
            current_date: "2026-09-25",
            integrity_sha256: integrity,
        }
    }

    #[test]
    fn validated_scope_can_enable_only_its_evidenced_desktop_operations() {
        let (evidence, integrity) = evidence_value();
        let validated =
            validate_desktop_evidence(&evidence.to_string(), &expectation(&integrity)).unwrap();
        let scope = validated.eligibility();
        let input = DesktopEligibilityInput {
            profile: CompatibilityProfile::Desktop644,
            source_format: SourceFormat::DesktopDelphiComponentStream,
            layout: DesktopLayout::SupportedComponentStream,
            adaptations: &[DesktopAdaptation::LoadSpellingPatch],
            advancement: crate::compatibility::DesktopAdvancementProvenance::Unadvanced,
            passkey: 42,
            realm: "Synthetic Realm",
            endpoint: "http://synthetic.invalid/report?",
            account: "account",
            password: "password",
            encoding_supported: true,
            operation: DesktopOnlineOperation::ManualBrag,
        };
        assert_eq!(
            evaluate_desktop_eligibility(&input, Some(&scope)),
            DesktopEligibilityDecision::Eligible
        );
        let wrong_operation = DesktopEligibilityInput {
            operation: DesktopOnlineOperation::Guild,
            ..input
        };
        assert!(matches!(
            evaluate_desktop_eligibility(&wrong_operation, Some(&scope)),
            DesktopEligibilityDecision::Ineligible(_)
        ));
    }

    #[test]
    fn bundled_pemptus_records_are_independent_pinned_and_fail_closed_when_removed_or_stale() {
        let contract = crate::desktop_contract::PEMPTUS;
        let mut records = production_records();
        for operation in [
            DesktopOnlineOperation::ManualBrag,
            DesktopOnlineOperation::Motto,
        ] {
            let validated =
                select_desktop_evidence(&records, contract, operation, "2026-10-01").unwrap();
            assert_eq!(validated.eligibility().operations, [operation]);
            assert!(validated.guild_fingerprints().is_none());
            assert!(matches!(
                select_desktop_evidence(&records, contract, operation, "2027-10-02"),
                Err(DesktopEvidenceError::Stale),
            ));
        }
        records.retain(|record| {
            !(record.contract == contract && record.operation == DesktopOnlineOperation::ManualBrag)
        });
        assert!(matches!(
            select_desktop_evidence(
                &records,
                contract,
                DesktopOnlineOperation::ManualBrag,
                "2026-10-01"
            ),
            Err(DesktopEvidenceError::Unavailable),
        ));
        assert!(
            select_desktop_evidence(
                &records,
                contract,
                DesktopOnlineOperation::Motto,
                "2026-10-01"
            )
            .is_ok()
        );
        with_synthetic_desktop_evidence(vec![], || {
            for operation in [
                DesktopOnlineOperation::AutomaticLevel,
                DesktopOnlineOperation::AutomaticAct,
                DesktopOnlineOperation::ManualBrag,
                DesktopOnlineOperation::Motto,
                DesktopOnlineOperation::Guild,
            ] {
                assert!(production_desktop_evidence(contract, operation).is_err());
            }
        });
    }

    #[test]
    fn bundled_tampered_and_stale_evidence_are_handled_independently() {
        let (mut evidence, integrity) = evidence_value();
        assert!(
            production_desktop_evidence(
                crate::desktop_contract::SPOLTOG,
                DesktopOnlineOperation::ManualBrag,
            )
            .is_ok()
        );
        evidence["payload"]["classification"] = Value::String("cheater".to_owned());
        assert_eq!(
            validate_desktop_evidence(&evidence.to_string(), &expectation(&integrity)).unwrap_err(),
            DesktopEvidenceError::Integrity
        );

        let (mut stale, _) = evidence_value();
        stale["payload"]["validThrough"] = Value::String("2026-09-24".to_owned());
        let stale_integrity = payload_sha256(
            &serde_json::from_value::<DesktopEvidenceEnvelope>(stale.clone())
                .unwrap()
                .payload,
        )
        .unwrap();
        stale["integritySha256"] = Value::String(stale_integrity.clone());
        assert_eq!(
            validate_desktop_evidence(&stale.to_string(), &expectation(&stale_integrity))
                .unwrap_err(),
            DesktopEvidenceError::Stale
        );
    }

    #[test]
    fn wrong_profile_realm_operation_and_inconclusive_results_never_validate() {
        let cases = [
            (
                "/payload/targetProfile",
                Value::String("browser".to_owned()),
                DesktopEvidenceError::ProfileMismatch,
            ),
            (
                "/payload/realm",
                Value::String("Other Realm".to_owned()),
                DesktopEvidenceError::RealmMismatch,
            ),
            (
                "/payload/operations",
                json!(["guild"]),
                DesktopEvidenceError::OperationMismatch,
            ),
            (
                "/payload/status",
                Value::String("inconclusive".to_owned()),
                DesktopEvidenceError::Inconclusive,
            ),
        ];
        for (pointer, replacement, expected_error) in cases {
            let (mut evidence, _) = evidence_value();
            *evidence.pointer_mut(pointer).unwrap() = replacement;
            let envelope: DesktopEvidenceEnvelope =
                serde_json::from_value(evidence.clone()).unwrap();
            let integrity = payload_sha256(&envelope.payload).unwrap();
            evidence["integritySha256"] = Value::String(integrity.clone());
            assert_eq!(
                validate_desktop_evidence(&evidence.to_string(), &expectation(&integrity))
                    .unwrap_err(),
                expected_error
            );
        }
    }

    #[test]
    fn browser_evidence_and_changed_implementation_identity_are_rejected() {
        let browser = include_str!("../tests/fixtures/enrollment-conformance-evidence.json");
        assert!(matches!(
            validate_desktop_evidence(browser, &expectation(&"0".repeat(64))),
            Err(DesktopEvidenceError::Malformed | DesktopEvidenceError::Sensitive)
        ));

        let (mut evidence, _) = evidence_value();
        evidence["payload"]["implementationIdentity"] =
            Value::String("desktop-online-contract/changed".to_owned());
        let envelope: DesktopEvidenceEnvelope = serde_json::from_value(evidence.clone()).unwrap();
        let integrity = payload_sha256(&envelope.payload).unwrap();
        evidence["integritySha256"] = Value::String(integrity.clone());
        assert_eq!(
            validate_desktop_evidence(&evidence.to_string(), &expectation(&integrity)).unwrap_err(),
            DesktopEvidenceError::IdentityMismatch
        );
    }

    fn operation_payload(operation: DesktopOnlineOperation) -> DesktopEvidencePayload {
        let contract = crate::desktop_contract::PEMPTUS;
        DesktopEvidencePayload {
            realm: contract.realm.to_owned(),
            saved_endpoint: contract.saved_endpoint.to_owned(),
            verified_https_endpoint: contract.https_endpoint.to_owned(),
            credential_mode: contract.credential_mode,
            implementation_identity: contract.implementation_identity.to_owned(),
            import_path: DesktopEvidenceImportPath {
                adaptations: vec![],
                ..payload().import_path
            },
            operations: vec![operation],
            encoding: Some("ascii".to_owned()),
            accepted_cases: Some(DesktopEvidenceCase::required(operation).to_vec()),
            guild_responses: if operation == DesktopOnlineOperation::Guild {
                let mut guild = payload().guild_responses.unwrap();
                guild.change_accepted_fingerprint_sha256 = Some("4".repeat(64));
                Some(guild)
            } else {
                None
            },
            ..payload()
        }
    }

    fn operation_document(payload: DesktopEvidencePayload) -> (String, String) {
        let integrity = payload_sha256(&payload).unwrap();
        let content = serde_json::to_string(&DesktopEvidenceEnvelope {
            format: DESKTOP_OPERATION_EVIDENCE_FORMAT.to_owned(),
            payload,
            integrity_sha256: integrity.clone(),
        })
        .unwrap();
        (content, integrity)
    }

    fn validate_operation(
        payload: DesktopEvidencePayload,
        operation: DesktopOnlineOperation,
    ) -> Result<ValidatedDesktopEvidence, DesktopEvidenceError> {
        let (content, integrity) = operation_document(payload);
        select_desktop_evidence(
            &[DesktopEvidenceRecord {
                contract: crate::desktop_contract::PEMPTUS,
                operation,
                content: &content,
                integrity: &integrity,
            }],
            crate::desktop_contract::PEMPTUS,
            operation,
            "2026-09-25",
        )
    }

    #[test]
    fn operation_records_cover_each_mode_operation_and_spelling_equivalence_independently() {
        use crate::compatibility::DesktopAdvancementProvenance;
        for operation in [
            DesktopOnlineOperation::AutomaticLevel,
            DesktopOnlineOperation::AutomaticAct,
            DesktopOnlineOperation::ManualBrag,
            DesktopOnlineOperation::Motto,
            DesktopOnlineOperation::Guild,
        ] {
            let validated = validate_operation(operation_payload(operation), operation).unwrap();
            let scope = validated.eligibility();
            for adaptations in [&[][..], &[DesktopAdaptation::LoadSpellingPatch][..]] {
                let input = DesktopEligibilityInput {
                    profile: CompatibilityProfile::Desktop644,
                    source_format: SourceFormat::DesktopDelphiComponentStream,
                    layout: DesktopLayout::SupportedComponentStream,
                    adaptations,
                    advancement: DesktopAdvancementProvenance::Unadvanced,
                    passkey: 42,
                    realm: crate::desktop_contract::PEMPTUS.realm,
                    endpoint: crate::desktop_contract::PEMPTUS.saved_endpoint,
                    account: "",
                    password: "",
                    encoding_supported: true,
                    operation,
                };
                assert_eq!(
                    evaluate_desktop_eligibility(&input, Some(&scope)),
                    DesktopEligibilityDecision::Eligible
                );
                assert_eq!(
                    production_desktop_evidence(crate::desktop_contract::PEMPTUS, operation)
                        .is_ok(),
                    matches!(
                        operation,
                        DesktopOnlineOperation::ManualBrag | DesktopOnlineOperation::Motto
                    )
                );
                assert!(
                    production_desktop_evidence(crate::desktop_contract::SPOLTOG, operation)
                        .is_ok()
                );
                assert_ne!(
                    evaluate_desktop_eligibility(
                        &DesktopEligibilityInput {
                            account: "a",
                            password: "b",
                            ..input
                        },
                        Some(&scope),
                    ),
                    DesktopEligibilityDecision::Eligible
                );
            }
        }
    }

    #[test]
    fn operation_scope_acceptance_identity_and_cleanup_cannot_be_relabeled() {
        let operation = DesktopOnlineOperation::ManualBrag;
        for mutate in [
            |p: &mut DesktopEvidencePayload| p.realm = "Spoltog".to_owned(),
            |p: &mut DesktopEvidencePayload| p.saved_endpoint.push_str("alias"),
            |p: &mut DesktopEvidencePayload| p.verified_https_endpoint.push_str("?query"),
            |p: &mut DesktopEvidencePayload| {
                p.credential_mode = DesktopCredentialMode::AccountPassword
            },
            |p: &mut DesktopEvidencePayload| p.implementation_identity.push_str("/changed"),
            |p: &mut DesktopEvidencePayload| p.source_identity.commit = "0".repeat(40),
            |p: &mut DesktopEvidencePayload| p.encoding = Some("utf8".to_owned()),
            |p: &mut DesktopEvidencePayload| p.operations.push(DesktopOnlineOperation::Motto),
            |p: &mut DesktopEvidencePayload| p.accepted_cases = Some(vec![]),
            |p: &mut DesktopEvidencePayload| p.cleanup_complete = false,
            |p: &mut DesktopEvidencePayload| p.classification = "cheater".to_owned(),
            |p: &mut DesktopEvidencePayload| p.status = "inconclusive".to_owned(),
            |p: &mut DesktopEvidencePayload| p.valid_through = "2026-09-24".to_owned(),
            |p: &mut DesktopEvidencePayload| p.observed_on = "2026-09-26".to_owned(),
            |p: &mut DesktopEvidencePayload| p.observed_on = "2026-09-00".to_owned(),
            |p: &mut DesktopEvidencePayload| p.valid_through = "2027-02-29".to_owned(),
            |p: &mut DesktopEvidencePayload| {
                p.import_path.adaptations = vec![DesktopAdaptation::LegacyPrologue62]
            },
        ] {
            let mut payload = operation_payload(operation);
            mutate(&mut payload);
            assert!(validate_operation(payload, operation).is_err());
        }
        let (mut content, integrity) = operation_document(operation_payload(operation));
        content = content.replace("\"normal\"", "\"cheater\"");
        assert_eq!(
            select_desktop_evidence(
                &[DesktopEvidenceRecord {
                    contract: crate::desktop_contract::PEMPTUS,
                    operation,
                    content: &content,
                    integrity: &integrity,
                }],
                crate::desktop_contract::PEMPTUS,
                operation,
                "2026-09-25"
            )
            .unwrap_err(),
            DesktopEvidenceError::Integrity
        );
    }

    #[test]
    fn invalid_guild_record_does_not_poison_an_independent_manual_record() {
        let contract = crate::desktop_contract::PEMPTUS;
        let (manual, manual_integrity) =
            operation_document(operation_payload(DesktopOnlineOperation::ManualBrag));
        let (guild, guild_integrity) = operation_document(DesktopEvidencePayload {
            guild_responses: None,
            ..operation_payload(DesktopOnlineOperation::Guild)
        });
        let records = [
            DesktopEvidenceRecord {
                contract,
                operation: DesktopOnlineOperation::ManualBrag,
                content: &manual,
                integrity: &manual_integrity,
            },
            DesktopEvidenceRecord {
                contract,
                operation: DesktopOnlineOperation::Guild,
                content: &guild,
                integrity: &guild_integrity,
            },
        ];
        let manual = select_desktop_evidence(
            &records,
            contract,
            DesktopOnlineOperation::ManualBrag,
            "2026-09-25",
        )
        .unwrap();
        assert!(manual.guild_fingerprints().is_none());
        assert!(
            select_desktop_evidence(
                &records,
                contract,
                DesktopOnlineOperation::Guild,
                "2026-09-25"
            )
            .is_err()
        );
        assert!(
            select_desktop_evidence(
                &records,
                contract,
                DesktopOnlineOperation::AutomaticAct,
                "2026-09-25"
            )
            .is_err()
        );
        let mut ambiguous = operation_payload(DesktopOnlineOperation::Guild);
        let guild = ambiguous.guild_responses.as_mut().unwrap();
        guild.rejected_fingerprint_sha256 = guild.join_accepted_fingerprint_sha256.clone();
        assert!(validate_operation(ambiguous, DesktopOnlineOperation::Guild).is_err());
    }
}
