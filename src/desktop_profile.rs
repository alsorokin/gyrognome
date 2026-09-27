use ring::digest::{Context, SHA256};
use thiserror::Error;

use crate::{
    compatibility::CompatibilityProfile, desktop_save::DesktopValidatedProfile,
    desktop_transport::DesktopTransportError,
};

pub const MAX_DESKTOP_GUILD_RESPONSE_BYTES: usize = 16 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DesktopGuildOperation {
    JoinOrChange,
    Leave,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DesktopGuildOutcome {
    Accepted,
    Rejected,
    Indeterminate,
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DesktopMottoOutcome {
    Delivered,
    EndpointRejected,
    DeliveryFailed,
    Cancelled,
}

#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
pub enum DesktopProfileError {
    #[error("desktop profile text uses unsupported non-ASCII encoding")]
    UnsupportedEncoding,
    #[error("desktop guild evidence is unavailable")]
    EvidenceUnavailable,
    #[error("desktop guild evidence profile does not match")]
    ProfileMismatch,
    #[error("desktop guild evidence realm does not match")]
    RealmMismatch,
    #[error("desktop guild evidence operation does not match")]
    OperationMismatch,
}

pub struct DesktopGuildEvidence<'a> {
    pub profile: CompatibilityProfile,
    pub realm: &'a str,
    pub operation: DesktopGuildOperation,
    pub accepted_fingerprint: &'a str,
    pub rejected_fingerprint: &'a str,
}

pub struct DesktopGuildResponseRules {
    accepted_fingerprint: String,
    rejected_fingerprint: Option<String>,
}

impl DesktopGuildResponseRules {
    pub fn production(
        realm: &str,
        operation: DesktopGuildOperation,
    ) -> Result<Self, DesktopProfileError> {
        if realm != "Spoltog" {
            return Err(DesktopProfileError::RealmMismatch);
        }
        let Some((join_accepted, rejected, leave_accepted)) =
            crate::desktop_evidence::production_guild_response_fingerprints()
        else {
            return Err(DesktopProfileError::EvidenceUnavailable);
        };
        Ok(match operation {
            DesktopGuildOperation::JoinOrChange => Self {
                accepted_fingerprint: join_accepted.to_owned(),
                rejected_fingerprint: Some(rejected.to_owned()),
            },
            DesktopGuildOperation::Leave => Self {
                accepted_fingerprint: leave_accepted.to_owned(),
                rejected_fingerprint: None,
            },
        })
    }

    pub fn from_evidence(
        expected_realm: &str,
        expected_operation: DesktopGuildOperation,
        evidence: &DesktopGuildEvidence<'_>,
    ) -> Result<Self, DesktopProfileError> {
        if evidence.profile != CompatibilityProfile::Desktop644 {
            return Err(DesktopProfileError::ProfileMismatch);
        }
        if evidence.realm != expected_realm {
            return Err(DesktopProfileError::RealmMismatch);
        }
        if evidence.operation != expected_operation {
            return Err(DesktopProfileError::OperationMismatch);
        }
        if evidence.accepted_fingerprint.len() != 64
            || evidence.rejected_fingerprint.len() != 64
            || !evidence
                .accepted_fingerprint
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit())
            || !evidence
                .rejected_fingerprint
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit())
        {
            return Err(DesktopProfileError::EvidenceUnavailable);
        }
        Ok(Self {
            accepted_fingerprint: evidence.accepted_fingerprint.to_ascii_lowercase(),
            rejected_fingerprint: Some(evidence.rejected_fingerprint.to_ascii_lowercase()),
        })
    }

    pub fn classify(
        &self,
        status: u16,
        body: &[u8],
        prior: &str,
        submitted: &str,
    ) -> DesktopGuildOutcome {
        if !(200..300).contains(&status) || body.len() > MAX_DESKTOP_GUILD_RESPONSE_BYTES {
            return DesktopGuildOutcome::Indeterminate;
        }
        let Ok(body) = std::str::from_utf8(body) else {
            return DesktopGuildOutcome::Indeterminate;
        };
        let fingerprint = desktop_guild_fingerprint(body, prior, submitted);
        if fingerprint == self.accepted_fingerprint {
            DesktopGuildOutcome::Accepted
        } else if self
            .rejected_fingerprint
            .as_ref()
            .is_some_and(|rejected| fingerprint == *rejected)
        {
            DesktopGuildOutcome::Rejected
        } else {
            DesktopGuildOutcome::Indeterminate
        }
    }
}

pub fn apply_desktop_motto_action(
    profile: &mut DesktopValidatedProfile,
    submitted: Option<&str>,
    deliver_once: impl FnOnce(&DesktopValidatedProfile) -> Result<u16, DesktopTransportError>,
) -> Result<DesktopMottoOutcome, DesktopProfileError> {
    let Some(submitted) = submitted else {
        return Ok(DesktopMottoOutcome::Cancelled);
    };
    if !submitted.is_ascii() {
        return Err(DesktopProfileError::UnsupportedEncoding);
    }
    profile.motto = submitted.to_owned();
    Ok(match deliver_once(profile) {
        Ok(200..=299) => DesktopMottoOutcome::Delivered,
        Ok(_) => DesktopMottoOutcome::EndpointRejected,
        Err(_) => DesktopMottoOutcome::DeliveryFailed,
    })
}

pub fn apply_desktop_guild_action(
    profile: &mut DesktopValidatedProfile,
    submitted: Option<&str>,
    classify_once: impl FnOnce(&str, &str) -> DesktopGuildOutcome,
) -> Result<DesktopGuildOutcome, DesktopProfileError> {
    let Some(submitted) = submitted else {
        return Ok(DesktopGuildOutcome::Cancelled);
    };
    if !submitted.is_ascii() {
        return Err(DesktopProfileError::UnsupportedEncoding);
    }
    let outcome = classify_once(&profile.guild, submitted);
    if outcome == DesktopGuildOutcome::Accepted {
        profile.guild = submitted.to_owned();
    }
    Ok(outcome)
}

fn desktop_guild_fingerprint(body: &str, prior: &str, submitted: &str) -> String {
    let mut designations = [prior, submitted];
    designations.sort_by_key(|value| std::cmp::Reverse(value.len()));
    let mut remaining = body;
    let mut digest = Context::new(&SHA256);
    while !remaining.is_empty() {
        if let Some(value) = designations
            .iter()
            .find(|value| !value.is_empty() && remaining.starts_with(**value))
        {
            digest.update(b"<desktop-guild-designation>");
            remaining = &remaining[value.len()..];
        } else {
            let length = remaining
                .chars()
                .next()
                .expect("nonempty desktop guild response remainder")
                .len_utf8();
            digest.update(remaining[..length].as_bytes());
            remaining = &remaining[length..];
        }
    }
    digest
        .finish()
        .as_ref()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use super::*;

    fn profile() -> DesktopValidatedProfile {
        DesktopValidatedProfile {
            motto: "Old motto".to_owned(),
            guild: "Old Guild".to_owned(),
        }
    }

    fn rules(operation: DesktopGuildOperation) -> DesktopGuildResponseRules {
        let accepted = desktop_guild_fingerprint("Accepted New Guild", "Old Guild", "New Guild");
        let rejected = desktop_guild_fingerprint("Rejected New Guild", "Old Guild", "New Guild");
        DesktopGuildResponseRules::from_evidence(
            "Synthetic Realm",
            operation,
            &DesktopGuildEvidence {
                profile: CompatibilityProfile::Desktop644,
                realm: "Synthetic Realm",
                operation,
                accepted_fingerprint: &accepted,
                rejected_fingerprint: &rejected,
            },
        )
        .unwrap()
    }

    #[test]
    fn desktop_guild_rules_are_bounded_and_scoped_to_spoltog() {
        assert!(matches!(
            DesktopGuildResponseRules::production(
                "Synthetic Realm",
                DesktopGuildOperation::JoinOrChange
            ),
            Err(DesktopProfileError::RealmMismatch)
        ));
        assert!(
            DesktopGuildResponseRules::production("Spoltog", DesktopGuildOperation::JoinOrChange)
                .is_ok()
        );
        assert!(
            DesktopGuildResponseRules::production("Spoltog", DesktopGuildOperation::Leave).is_ok()
        );
        let accepted = "a".repeat(64);
        let rejected = "b".repeat(64);
        let browser = DesktopGuildEvidence {
            profile: CompatibilityProfile::Browser,
            realm: "Synthetic Realm",
            operation: DesktopGuildOperation::JoinOrChange,
            accepted_fingerprint: &accepted,
            rejected_fingerprint: &rejected,
        };
        assert!(matches!(
            DesktopGuildResponseRules::from_evidence(
                "Synthetic Realm",
                DesktopGuildOperation::JoinOrChange,
                &browser
            ),
            Err(DesktopProfileError::ProfileMismatch)
        ));
        let wrong_realm = DesktopGuildEvidence {
            profile: CompatibilityProfile::Desktop644,
            realm: "Other Realm",
            ..browser
        };
        assert!(matches!(
            DesktopGuildResponseRules::from_evidence(
                "Synthetic Realm",
                DesktopGuildOperation::JoinOrChange,
                &wrong_realm
            ),
            Err(DesktopProfileError::RealmMismatch)
        ));
        let wrong_operation = DesktopGuildEvidence {
            profile: CompatibilityProfile::Desktop644,
            realm: "Synthetic Realm",
            operation: DesktopGuildOperation::Leave,
            ..browser
        };
        assert!(matches!(
            DesktopGuildResponseRules::from_evidence(
                "Synthetic Realm",
                DesktopGuildOperation::JoinOrChange,
                &wrong_operation
            ),
            Err(DesktopProfileError::OperationMismatch)
        ));
    }

    #[test]
    fn unknown_oversized_rejected_and_failed_guild_results_preserve_membership() {
        let rules = rules(DesktopGuildOperation::JoinOrChange);
        let cases = [
            rules.classify(200, b"Rejected New Guild", "Old Guild", "New Guild"),
            rules.classify(200, b"Unknown New Guild", "Old Guild", "New Guild"),
            rules.classify(
                200,
                &vec![b'x'; MAX_DESKTOP_GUILD_RESPONSE_BYTES + 1],
                "Old Guild",
                "New Guild",
            ),
            rules.classify(503, b"Accepted New Guild", "Old Guild", "New Guild"),
        ];
        for outcome in cases {
            let mut profile = profile();
            assert_ne!(outcome, DesktopGuildOutcome::Accepted);
            assert_eq!(
                apply_desktop_guild_action(&mut profile, Some("New Guild"), |_, _| outcome)
                    .unwrap(),
                outcome
            );
            assert_eq!(profile.guild, "Old Guild");
        }
    }

    #[test]
    fn accepted_guild_mutates_once_and_cancel_or_non_ascii_does_nothing() {
        let calls = Cell::new(0);
        let mut profile = profile();
        assert_eq!(
            apply_desktop_guild_action(&mut profile, Some("New Guild"), |_, _| {
                calls.set(calls.get() + 1);
                DesktopGuildOutcome::Accepted
            })
            .unwrap(),
            DesktopGuildOutcome::Accepted
        );
        assert_eq!(calls.get(), 1);
        assert_eq!(profile.guild, "New Guild");

        assert_eq!(
            apply_desktop_guild_action(&mut profile, None, |_, _| {
                panic!("cancelled guild action attempted delivery")
            })
            .unwrap(),
            DesktopGuildOutcome::Cancelled
        );
        assert_eq!(profile.guild, "New Guild");
        assert!(matches!(
            apply_desktop_guild_action(&mut profile, Some("Gnomé"), |_, _| {
                panic!("unsupported guild action attempted delivery")
            }),
            Err(DesktopProfileError::UnsupportedEncoding)
        ));
        assert_eq!(profile.guild, "New Guild");
    }

    #[test]
    fn motto_persists_before_one_delivery_and_survives_failure() {
        for (delivery, expected) in [
            (Ok(204), DesktopMottoOutcome::Delivered),
            (Ok(500), DesktopMottoOutcome::EndpointRejected),
            (
                Err(DesktopTransportError::DeliveryFailed),
                DesktopMottoOutcome::DeliveryFailed,
            ),
        ] {
            let calls = Cell::new(0);
            let mut profile = profile();
            let outcome =
                apply_desktop_motto_action(&mut profile, Some("Selected motto"), |persisted| {
                    calls.set(calls.get() + 1);
                    assert_eq!(persisted.motto, "Selected motto");
                    delivery
                })
                .unwrap();
            assert_eq!(outcome, expected);
            assert_eq!(calls.get(), 1);
            assert_eq!(profile.motto, "Selected motto");
        }
    }

    #[test]
    fn cancelled_or_non_ascii_motto_never_mutates_or_delivers() {
        let mut profile = profile();
        assert_eq!(
            apply_desktop_motto_action(&mut profile, None, |_| {
                panic!("cancelled motto action attempted delivery")
            })
            .unwrap(),
            DesktopMottoOutcome::Cancelled
        );
        assert_eq!(profile.motto, "Old motto");
        assert!(matches!(
            apply_desktop_motto_action(&mut profile, Some("Gnomé"), |_| {
                panic!("unsupported motto action attempted delivery")
            }),
            Err(DesktopProfileError::UnsupportedEncoding)
        ));
        assert_eq!(profile.motto, "Old motto");
    }
}
