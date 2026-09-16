use std::path::Path;

use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum FixtureSafetyError {
    #[error("fixture path must not use the .pqw player-save extension")]
    PlayerSave,
    #[error("fixture must not contain a full signed leaderboard request")]
    SignedRequest,
    #[error("fixture must not contain a browser profile path")]
    BrowserProfile,
    #[error("checkpoint fixture must not contain an online passkey")]
    Passkey,
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
        && content.to_ascii_lowercase().contains("passkey")
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
    }
}
