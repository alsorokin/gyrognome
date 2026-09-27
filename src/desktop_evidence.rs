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
pub const DESKTOP_ONLINE_IMPLEMENTATION_ID: &str = "desktop-online-contract/v1";
pub const PRODUCTION_DESKTOP_EVIDENCE_INTEGRITY: &str =
    "edd7f9aa16b5be4f4a695a12d417e3505e55aeb5d5d68b180b8d41f16a3860f1";
const PRODUCTION_DESKTOP_EVIDENCE: &str =
    include_str!("../tests/fixtures/desktop-online-evidence.json");

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
    guild_responses: DesktopGuildResponseEvidence,
    classification: String,
    cleanup_complete: bool,
    observed_on: String,
    valid_through: String,
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
    if envelope.format != DESKTOP_EVIDENCE_FORMAT {
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
    if envelope.payload.valid_through.as_str() < expected.current_date {
        return Err(DesktopEvidenceError::Stale);
    }
    if envelope.payload.source_identity.tag != SOURCE_TAG
        || envelope.payload.source_identity.commit != SOURCE_COMMIT
        || envelope.payload.source_identity.main_pas_sha256 != MAIN_PAS_SHA256
        || envelope.payload.source_identity.config_dfm_sha256 != CONFIG_DFM_SHA256
        || envelope.payload.implementation_identity != DESKTOP_ONLINE_IMPLEMENTATION_ID
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
    if envelope.payload.operations.is_empty()
        || envelope.payload.operations.len() > 5
        || envelope.payload.guild_responses.normalization != DESKTOP_RESPONSE_FINGERPRINT_VERSION
        || ![
            &envelope
                .payload
                .guild_responses
                .join_accepted_fingerprint_sha256,
            &envelope.payload.guild_responses.rejected_fingerprint_sha256,
            &envelope
                .payload
                .guild_responses
                .leave_accepted_fingerprint_sha256,
        ]
        .into_iter()
        .all(|value| valid_sha256(value))
    {
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

pub(crate) fn production_desktop_evidence_is_valid() -> bool {
    let current_date = current_utc_date();
    [
        DesktopOnlineOperation::AutomaticLevel,
        DesktopOnlineOperation::AutomaticAct,
        DesktopOnlineOperation::ManualBrag,
        DesktopOnlineOperation::Motto,
        DesktopOnlineOperation::Guild,
    ]
    .into_iter()
    .all(|operation| {
        validate_production_desktop_evidence(&DesktopEvidenceExpectation {
            realm: "Spoltog",
            saved_endpoint: "http://progressquest.com/spoltog.php?",
            verified_https_endpoint: "https://progressquest.com/spoltog.php",
            operation,
            current_date: &current_date,
            integrity_sha256: PRODUCTION_DESKTOP_EVIDENCE_INTEGRITY,
        })
        .is_ok()
    })
}

pub(crate) fn production_guild_response_fingerprints()
-> Option<(&'static str, &'static str, &'static str, &'static str)> {
    production_desktop_evidence_is_valid().then_some((
        DESKTOP_RESPONSE_FINGERPRINT_VERSION,
        "f5a48b1c0f296b4e391d0f26736ee902b56daed589064898dd7c2f9279d6c115",
        "3d01c41d894571762aed5d08d4427e2b4d33a9f4a15e8153c72f1c3152984b71",
        "8dd3a69a69f617062731571d8dace6974aecd303eb4320af84d2308fd75d4667",
    ))
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
    value.len() == 10
        && value.as_bytes()[4] == b'-'
        && value.as_bytes()[7] == b'-'
        && value
            .bytes()
            .enumerate()
            .all(|(index, byte)| index == 4 || index == 7 || byte.is_ascii_digit())
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
            guild_responses: DesktopGuildResponseEvidence {
                normalization: DESKTOP_RESPONSE_FINGERPRINT_VERSION.to_owned(),
                join_accepted_fingerprint_sha256: "1".repeat(64),
                rejected_fingerprint_sha256: "2".repeat(64),
                leave_accepted_fingerprint_sha256: "3".repeat(64),
            },
            classification: "normal".to_owned(),
            cleanup_complete: true,
            observed_on: "2026-09-20".to_owned(),
            valid_through: "2026-10-20".to_owned(),
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
    fn bundled_tampered_and_stale_evidence_are_handled_independently() {
        let (mut evidence, integrity) = evidence_value();
        assert!(production_desktop_evidence_is_valid());
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
}
