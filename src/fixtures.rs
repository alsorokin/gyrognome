use std::{fs, path::Path};

use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum FixtureSafetyError {
    #[error("fixture path must not use the .pqw player-save extension")]
    PlayerSave,
    #[error("fixture must not contain a full signed leaderboard request")]
    SignedRequest,
    #[error("fixture must not contain a browser profile path")]
    BrowserProfile,
    #[error("fixture must not contain a raw browser save or response body")]
    RawBrowserData,
    #[error("checkpoint fixture must not contain an online passkey")]
    Passkey,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum EnrollmentEvidenceError {
    #[error("enrollment conformance evidence is unavailable")]
    Unavailable,
    #[error("enrollment conformance evidence is malformed")]
    Malformed,
    #[error("enrollment conformance evidence contains sensitive data")]
    Sensitive,
    #[error("enrollment conformance evidence is not a complete passing live result")]
    NotLiveOrPassing,
    #[error("enrollment conformance evidence is incomplete")]
    Incomplete,
}

pub fn validate_enrollment_evidence(content: &str) -> Result<(), EnrollmentEvidenceError> {
    if validate_fixture(Path::new("enrollment-conformance-evidence.json"), content).is_err() {
        return Err(EnrollmentEvidenceError::Sensitive);
    }
    let evidence: Value =
        serde_json::from_str(content).map_err(|_| EnrollmentEvidenceError::Malformed)?;
    if evidence["format"] != "gyrognome-disposable-conformance/v2" {
        return Err(EnrollmentEvidenceError::Malformed);
    }
    if evidence["mode"] != "submission-enabled"
        || evidence["source"]["client"]
            .as_str()
            .is_none_or(str::is_empty)
        || evidence["source"]["revision"]
            .as_str()
            .is_none_or(str::is_empty)
        || evidence["source"]["content_sha256"]
            .as_str()
            .is_none_or(str::is_empty)
        || evidence["summary"]["total"] != evidence["summary"]["passed"]
        || !evidence["summary"]["failed"]
            .as_array()
            .is_some_and(Vec::is_empty)
    {
        return Err(EnrollmentEvidenceError::NotLiveOrPassing);
    }
    let enrollment = &evidence["enrollment"];
    if enrollment["successfulCreation"]["outcome"] != "success"
        || enrollment["successfulCreation"]["creation"].is_null()
        || enrollment["successfulCreation"]["initialReport"].is_null()
        || enrollment["successfulCreation"]["order"] != "create-before-initial-report"
        || enrollment["successfulCreation"]["pass"] != true
        || enrollment["duplicateName"]["outcome"] != "rejected"
        || enrollment["duplicateName"]["creation"].is_null()
        || enrollment["duplicateName"]["creationAttempts"] != 1
        || enrollment["duplicateName"]["initialReportAttempts"] != 0
        || enrollment["duplicateName"]["additionalOnlineIdentity"] != false
        || enrollment["duplicateName"]["pass"] != true
        || enrollment["interruptedEnrollment"]["outcome"] != "unconfirmed"
        || enrollment["interruptedEnrollment"]["creations"]
            .as_array()
            .is_none_or(Vec::is_empty)
        || enrollment["interruptedEnrollment"]["initialReportAttempts"] != 0
        || enrollment["interruptedEnrollment"]["pass"] != true
    {
        return Err(EnrollmentEvidenceError::Incomplete);
    }
    Ok(())
}

pub fn validate_enrollment_evidence_file(path: &Path) -> Result<(), EnrollmentEvidenceError> {
    let content = fs::read_to_string(path).map_err(|_| EnrollmentEvidenceError::Unavailable)?;
    validate_enrollment_evidence(&content)
}

pub fn validate_bundled_enrollment_evidence() -> Result<(), EnrollmentEvidenceError> {
    validate_enrollment_evidence(include_str!(
        "../tests/fixtures/enrollment-conformance-evidence.json"
    ))
}

pub fn validate_fixture(path: &Path, content: &str) -> Result<(), FixtureSafetyError> {
    if path
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("pqw"))
    {
        return Err(FixtureSafetyError::PlayerSave);
    }
    if content.contains("cmd=") && content.contains("&p=") {
        return Err(FixtureSafetyError::SignedRequest);
    }
    if content.contains(".playwright-mcp")
        || content.contains("Default/")
        || content.contains("Chrome/User Data")
        || content.contains("user-data-dir")
    {
        return Err(FixtureSafetyError::BrowserProfile);
    }
    let lower = content.to_ascii_lowercase();
    if lower.contains("\"response\"")
        || lower.contains("\"response_body\"")
        || lower.contains("\"raw_save\"")
        || lower.contains("\"profile\"")
    {
        return Err(FixtureSafetyError::RawBrowserData);
    }
    let is_trace_or_experiment =
        path.file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| {
                let name = name.to_ascii_lowercase();
                name.contains("trace")
                    || name.contains("experiment")
                    || name.contains("evidence")
                    || name.contains("diagnostic")
            });
    if (path
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.starts_with("checkpoint-"))
        || is_trace_or_experiment)
        && lower.contains("passkey")
    {
        return Err(FixtureSafetyError::Passkey);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    #[test]
    fn rejects_prohibited_fixture_content() {
        assert_eq!(
            validate_fixture(Path::new("reference.pqw"), "{}"),
            Err(FixtureSafetyError::PlayerSave)
        );
        assert_eq!(
            validate_fixture(Path::new("request.txt"), "cmd=b&t=l&p=123"),
            Err(FixtureSafetyError::SignedRequest)
        );
        assert_eq!(
            validate_fixture(Path::new("profile.txt"), ".playwright-mcp/Default"),
            Err(FixtureSafetyError::BrowserProfile)
        );
        assert_eq!(
            validate_fixture(Path::new("checkpoint-timing.json"), r#"{"passkey": 1}"#),
            Err(FixtureSafetyError::Passkey)
        );
        assert_eq!(
            validate_fixture(Path::new("report-trace.json"), r#"{"passkey": 1}"#),
            Err(FixtureSafetyError::Passkey)
        );
        assert_eq!(
            validate_fixture(
                Path::new("experiment-evidence.json"),
                r#"{"profile": "Chrome/User Data"}"#
            ),
            Err(FixtureSafetyError::BrowserProfile)
        );
        assert_eq!(
            validate_fixture(
                Path::new("enrollment-evidence.json"),
                r#"{"response_body": "unsafe"}"#
            ),
            Err(FixtureSafetyError::RawBrowserData)
        );
    }

    #[test]
    fn enrollment_evidence_gate_is_credential_free_and_complete() {
        let valid = include_str!("../tests/fixtures/enrollment-conformance-evidence.json");
        assert!(validate_enrollment_evidence(valid).is_ok());
        assert_eq!(
            validate_enrollment_evidence(r#"{"passkey": 1}"#),
            Err(EnrollmentEvidenceError::Sensitive)
        );
        assert_eq!(
            validate_enrollment_evidence(r#"{"format":"gyrognome-disposable-conformance/v2"}"#),
            Err(EnrollmentEvidenceError::NotLiveOrPassing)
        );
        assert_eq!(
            validate_enrollment_evidence_file(Path::new("target/no-evidence.json")),
            Err(EnrollmentEvidenceError::Unavailable)
        );
    }
}
