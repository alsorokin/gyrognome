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

#[derive(Debug, Error, PartialEq, Eq)]
pub enum GuildEvidenceError {
    #[error("guild conformance evidence is unavailable")]
    Unavailable,
    #[error("guild conformance evidence is malformed")]
    Malformed,
    #[error("guild conformance evidence contains sensitive data")]
    Sensitive,
    #[error("guild conformance evidence is not a complete passing live result")]
    NotLiveOrPassing,
    #[error("guild conformance evidence is incomplete")]
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

pub fn validate_guild_evidence(content: &str) -> Result<(), GuildEvidenceError> {
    validate_enrollment_evidence(content).map_err(|error| match error {
        EnrollmentEvidenceError::Unavailable => GuildEvidenceError::Unavailable,
        EnrollmentEvidenceError::Malformed => GuildEvidenceError::Malformed,
        EnrollmentEvidenceError::Sensitive => GuildEvidenceError::Sensitive,
        EnrollmentEvidenceError::NotLiveOrPassing => GuildEvidenceError::NotLiveOrPassing,
        EnrollmentEvidenceError::Incomplete => GuildEvidenceError::Incomplete,
    })?;
    let evidence: Value =
        serde_json::from_str(content).map_err(|_| GuildEvidenceError::Malformed)?;
    let guild = &evidence["guild"];
    if !exact_keys(
        guild,
        &[
            "normalization",
            "nonEmpty",
            "invalid",
            "empty",
            "order",
            "cancellation",
            "cleanup",
            "classification",
            "pass",
        ],
    ) || guild["normalization"] != "guild-designations-sha256/v1"
        || guild["pass"] != true
        || !valid_guild_submission(&guild["nonEmpty"], "accepted", false)
        || !valid_guild_submission(&guild["invalid"], "rejected", false)
        || !valid_guild_submission(&guild["empty"], "accepted", true)
        || guild["invalid"]["outcome"]["fingerprint"] == guild["nonEmpty"]["outcome"]["fingerprint"]
        || guild["invalid"]["outcome"]["fingerprint"] == guild["empty"]["outcome"]["fingerprint"]
        || guild["order"] != "non-empty-before-invalid-before-empty"
        || !exact_keys(&guild["cancellation"], &["requestAttempts", "pass"])
        || guild["cancellation"]["requestAttempts"] != 0
        || guild["cancellation"]["pass"] != true
        || !exact_keys(&guild["cleanup"], &["browserGuildEmpty", "pass"])
        || guild["cleanup"]["browserGuildEmpty"] != true
        || guild["cleanup"]["pass"] != true
        || !exact_keys(&guild["classification"], &["classification", "attempts"])
        || guild["classification"]["classification"] != "normal"
        || !guild["classification"]["attempts"]
            .as_array()
            .is_some_and(|attempts| {
                attempts.last().is_some_and(|value| value == "normal")
                    && attempts.iter().all(|value| {
                        matches!(value.as_str(), Some("normal" | "not-found" | "error"))
                    })
            })
        || !valid_guild_report_scenarios(&evidence)
    {
        return Err(GuildEvidenceError::Incomplete);
    }
    Ok(())
}

pub fn validate_guild_evidence_file(path: &Path) -> Result<(), GuildEvidenceError> {
    let content = fs::read_to_string(path).map_err(|_| GuildEvidenceError::Unavailable)?;
    validate_guild_evidence(&content)
}

pub fn validate_bundled_guild_evidence() -> Result<(), GuildEvidenceError> {
    validate_guild_evidence(include_str!(
        "../tests/fixtures/enrollment-conformance-evidence.json"
    ))
}

fn exact_keys(value: &Value, keys: &[&str]) -> bool {
    value.as_object().is_some_and(|object| {
        object.len() == keys.len() && keys.iter().all(|key| object.contains_key(*key))
    })
}

fn valid_guild_report_scenarios(evidence: &Value) -> bool {
    let ids = [
        "initial-load",
        "pause",
        "restart",
        "delayed-callback",
        "task-completion",
        "level-up",
        "act-completion",
        "manual-brag",
        "motto-change",
    ];
    evidence["summary"]["total"] == ids.len()
        && evidence["scenarios"].as_array().is_some_and(|scenarios| {
            scenarios.len() == ids.len()
                && scenarios.iter().zip(ids).all(|(scenario, id)| {
                    scenario["id"] == id
                        && scenario["pass"] == true
                        && scenario["classification"]["classification"] == "normal"
                        && scenario["differences"]
                            .as_array()
                            .is_some_and(Vec::is_empty)
                })
        })
}

fn valid_guild_submission(submission: &Value, category: &str, guild_empty: bool) -> bool {
    exact_keys(
        submission,
        &["request", "outcome", "browser", "serverVerified", "pass"],
    ) && submission["pass"] == true
        && submission["serverVerified"] == true
        && exact_keys(
            &submission["request"],
            &["endpoint", "method", "operation", "fields"],
        )
        && submission["request"]["endpoint"] == "https://progressquest.com/alpaquil.php"
        && submission["request"]["method"] == "GET"
        && submission["request"]["operation"] == "guild"
        && submission["request"]["fields"]
            == serde_json::json!(["cmd", "n", "r", "c", "l", "h", "rev", "guild"])
        && exact_keys(&submission["outcome"], &["category", "fingerprint"])
        && submission["outcome"]["category"] == category
        && submission["outcome"]["fingerprint"]
            .as_str()
            .is_some_and(|fingerprint| {
                fingerprint.len() == 64
                    && fingerprint
                        .bytes()
                        .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
            })
        && exact_keys(
            &submission["browser"],
            &["alerted", "navigated", "guildEmpty"],
        )
        && submission["browser"]["alerted"].is_boolean()
        && submission["browser"]["navigated"].is_boolean()
        && submission["browser"]["guildEmpty"] == guild_empty
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

    #[test]
    fn guild_evidence_gate_requires_complete_and_safe() {
        let mut evidence: Value = serde_json::from_str(include_str!(
            "../tests/fixtures/enrollment-conformance-evidence.json"
        ))
        .unwrap();
        assert_eq!(validate_bundled_guild_evidence(), Ok(()));
        evidence["guild"]["normalization"] = Value::String("guild-designations-sha256/v1".into());
        evidence["guild"]["invalid"] = evidence["guild"]["nonEmpty"].clone();
        for (key, category, byte) in [
            ("nonEmpty", "accepted", "a"),
            ("invalid", "rejected", "b"),
            ("empty", "accepted", "c"),
        ] {
            evidence["guild"][key]["outcome"] = serde_json::json!({
                "category": category, "fingerprint": byte.repeat(64)
            });
            evidence["guild"][key]["serverVerified"] = Value::Bool(true);
        }
        evidence["guild"]["order"] = Value::String("non-empty-before-invalid-before-empty".into());
        assert!(validate_guild_evidence(&evidence.to_string()).is_ok());

        fn collect_paths(value: &Value, path: String, paths: &mut Vec<String>) {
            paths.push(path.clone());
            if let Some(object) = value.as_object() {
                for (key, child) in object {
                    collect_paths(child, format!("{path}/{key}"), paths);
                }
            }
        }
        let mut paths = Vec::new();
        collect_paths(&evidence["guild"], "/guild".into(), &mut paths);
        for path in paths {
            for replacement in [
                None,
                Some(Value::Null),
                Some(Value::String("malformed".into())),
            ] {
                let mut invalid = evidence.clone();
                if let Some(value) = replacement {
                    *invalid.pointer_mut(&path).unwrap() = value;
                } else {
                    let (parent, key) = path.rsplit_once('/').unwrap();
                    invalid
                        .pointer_mut(parent)
                        .unwrap()
                        .as_object_mut()
                        .unwrap()
                        .remove(key);
                }
                assert_eq!(
                    validate_guild_evidence(&invalid.to_string()),
                    Err(GuildEvidenceError::Incomplete),
                    "{path}"
                );
            }
        }
        for (path, value) in [
            (
                "/guild/invalid/outcome/fingerprint",
                Value::String("a".repeat(64)),
            ),
            (
                "/guild/invalid/outcome/fingerprint",
                Value::String("c".repeat(64)),
            ),
            (
                "/guild/invalid/outcome/category",
                Value::String("accepted".into()),
            ),
            (
                "/guild/nonEmpty/outcome/fingerprint",
                Value::String("A".repeat(64)),
            ),
            ("/guild/cleanup/pass", Value::Bool(false)),
            (
                "/guild/classification/attempts",
                serde_json::json!(["cheater", "normal"]),
            ),
            ("/guild/cancellation/requestAttempts", Value::from(1)),
            (
                "/guild/invalid/request/fields",
                serde_json::json!(["cmd", "guild", "p"]),
            ),
            (
                "/guild/invalid/request/endpoint",
                Value::String("https://example.invalid/".into()),
            ),
            ("/guild/empty/browser/guildEmpty", Value::Bool(false)),
            ("/scenarios/0/pass", Value::Bool(false)),
            (
                "/scenarios/0/classification/classification",
                Value::String("cheater".into()),
            ),
            ("/scenarios/0/differences", serde_json::json!(["mismatch"])),
            ("/scenarios", serde_json::json!([])),
            ("/enrollment/duplicateName/pass", Value::Bool(false)),
        ] {
            let mut invalid = evidence.clone();
            *invalid.pointer_mut(path).unwrap() = value;
            assert_eq!(
                validate_guild_evidence(&invalid.to_string()),
                Err(GuildEvidenceError::Incomplete),
                "{path}"
            );
        }
        let mut sensitive = evidence.clone();
        sensitive["guild"]["invalid"]["outcome"]["designation"] = Value::String("private".into());
        assert_eq!(
            validate_guild_evidence(&sensitive.to_string()),
            Err(GuildEvidenceError::Incomplete)
        );

        let mut missing = evidence.clone();
        missing.as_object_mut().unwrap().remove("guild");
        assert_eq!(
            validate_guild_evidence(&missing.to_string()),
            Err(GuildEvidenceError::Incomplete)
        );
        assert_eq!(
            validate_guild_evidence("{"),
            Err(GuildEvidenceError::Malformed)
        );

        let mut malformed = evidence.clone();
        malformed["guild"]["nonEmpty"]["outcome"]["category"] = Value::String("unknown".to_owned());
        assert_eq!(
            validate_guild_evidence(&malformed.to_string()),
            Err(GuildEvidenceError::Incomplete)
        );

        for pointer in [
            "/guild/nonEmpty/pass",
            "/guild/empty/pass",
            "/guild/cleanup/pass",
        ] {
            let mut invalid = evidence.clone();
            *invalid.pointer_mut(pointer).unwrap() = Value::Bool(false);
            assert_eq!(
                validate_guild_evidence(&invalid.to_string()),
                Err(GuildEvidenceError::Incomplete)
            );
        }

        let mut evidence = evidence;
        evidence["guild"]["nonEmpty"]["response_body"] = Value::String("unsafe".to_owned());
        assert_eq!(
            validate_guild_evidence(&evidence.to_string()),
            Err(GuildEvidenceError::Sensitive)
        );
        assert_eq!(
            validate_guild_evidence_file(Path::new("target/no-guild-evidence.json")),
            Err(GuildEvidenceError::Unavailable)
        );
    }
}
