use thiserror::Error;

use crate::{
    compatibility::CompatibilityProfile,
    desktop_fingerprint::{DESKTOP_RESPONSE_FINGERPRINT_VERSION, DesktopGuildFingerprintValues},
    desktop_save::DesktopValidatedProfile,
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
    pub normalization: &'a str,
    pub accepted_fingerprint: &'a str,
    pub rejected_fingerprint: &'a str,
}

pub struct DesktopGuildResponseRules {
    verification: DesktopGuildVerification,
}

enum DesktopGuildVerification {
    Fingerprints {
        accepted_fingerprint: String,
        change_accepted_fingerprint: Option<String>,
        rejected_fingerprint: Option<String>,
    },
    PublicMembership,
}

impl DesktopGuildResponseRules {
    pub fn production(
        realm: &str,
        operation: DesktopGuildOperation,
    ) -> Result<Self, DesktopProfileError> {
        Self::production_for_import(realm, operation, &[])
    }

    pub(crate) fn production_for_import(
        realm: &str,
        operation: DesktopGuildOperation,
        adaptations: &[crate::compatibility::DesktopAdaptation],
    ) -> Result<Self, DesktopProfileError> {
        let contract = crate::desktop_contract::realm_contract(realm)
            .map_err(|_| DesktopProfileError::RealmMismatch)?;
        if contract == crate::desktop_contract::PEMPTUS
            && crate::desktop_eligibility::pemptus_import_path_supported(adaptations)
        {
            return Ok(Self {
                verification: DesktopGuildVerification::PublicMembership,
            });
        }
        let evidence = crate::desktop_evidence::production_desktop_evidence_for_import(
            contract,
            crate::desktop_eligibility::DesktopOnlineOperation::Guild,
            adaptations,
        )
        .map_err(|_| DesktopProfileError::EvidenceUnavailable)?;
        let fingerprints = evidence
            .guild_fingerprints()
            .ok_or(DesktopProfileError::EvidenceUnavailable)?;
        if fingerprints.normalization != DESKTOP_RESPONSE_FINGERPRINT_VERSION {
            return Err(DesktopProfileError::EvidenceUnavailable);
        }
        Ok(Self {
            verification: match operation {
                DesktopGuildOperation::JoinOrChange => DesktopGuildVerification::Fingerprints {
                    accepted_fingerprint: fingerprints.join,
                    change_accepted_fingerprint: fingerprints.change,
                    rejected_fingerprint: Some(fingerprints.rejected),
                },
                DesktopGuildOperation::Leave => DesktopGuildVerification::Fingerprints {
                    accepted_fingerprint: fingerprints.leave,
                    change_accepted_fingerprint: None,
                    rejected_fingerprint: None,
                },
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
        if evidence.normalization != DESKTOP_RESPONSE_FINGERPRINT_VERSION
            || evidence.accepted_fingerprint.len() != 64
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
            verification: DesktopGuildVerification::Fingerprints {
                accepted_fingerprint: evidence.accepted_fingerprint.to_ascii_lowercase(),
                change_accepted_fingerprint: None,
                rejected_fingerprint: Some(evidence.rejected_fingerprint.to_ascii_lowercase()),
            },
        })
    }

    pub(crate) fn classify(
        &self,
        status: u16,
        body: &[u8],
        dynamic_values: &DesktopGuildFingerprintValues<'_>,
    ) -> DesktopGuildOutcome {
        let DesktopGuildVerification::Fingerprints {
            accepted_fingerprint,
            change_accepted_fingerprint,
            rejected_fingerprint,
        } = &self.verification
        else {
            return DesktopGuildOutcome::Indeterminate;
        };
        if !(200..300).contains(&status) || body.len() > MAX_DESKTOP_GUILD_RESPONSE_BYTES {
            return DesktopGuildOutcome::Indeterminate;
        }
        let fingerprint = dynamic_values.fingerprint(body);
        if fingerprint == *accepted_fingerprint
            || change_accepted_fingerprint
                .as_ref()
                .is_some_and(|value| value == &fingerprint)
        {
            DesktopGuildOutcome::Accepted
        } else if rejected_fingerprint
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

pub(crate) fn public_guild(page: &str, name: &str) -> Option<Option<String>> {
    let mut observed = None;
    for row in page.split("<tr").skip(1) {
        let cells = row
            .split("<td")
            .skip(1)
            .map(|cell| cell.split_once('>').map(|(_, value)| value))
            .collect::<Option<Vec<_>>>()?;
        if cells.len() < 11 || html_unescape(cell_text(cells[1])?)? != name {
            continue;
        }
        if observed.is_some() {
            return None;
        }
        let guild_cell = cells[10];
        let guild_text = html_unescape(cell_text(guild_cell)?)?;
        let guild = if guild_cell.contains("guilds.php?id=") && !guild_text.is_empty() {
            Some(guild_text)
        } else if guild_text.is_empty() && !guild_cell.contains("guilds.php?id=") {
            None
        } else {
            return None;
        };
        observed = Some(guild);
    }
    observed
}

pub(crate) fn public_guild_matches(observed: Option<&str>, submitted: &str) -> bool {
    match observed {
        Some(guild) => !submitted.is_empty() && guild.eq_ignore_ascii_case(submitted),
        None => submitted.is_empty(),
    }
}

pub(crate) fn public_guild_for_realm(
    realm: &str,
    page: &str,
    name: &str,
) -> Option<Option<String>> {
    match realm {
        "Spoltog" => public_guild(page, name),
        "Pemptus" => pemptus_public_row(page, name).map(|(_, guild)| guild),
        _ => None,
    }
}

pub(crate) fn pemptus_public_row(page: &str, name: &str) -> Option<(Vec<String>, Option<String>)> {
    const HEADERS: [&str; 11] = [
        "Rank",
        "Name",
        "Race",
        "Class",
        "Level",
        "Prime Stat",
        "Plot Stage",
        "Prized Item",
        "Specialty",
        "Motto",
        "Guild",
    ];
    fn cells<'a>(row: &'a str, tag: &str) -> Option<Vec<&'a str>> {
        let marker = format!("<{tag}");
        row.split(&marker)
            .skip(1)
            .map(|cell| {
                let (attributes, text) = cell.split_once('>')?;
                if (!attributes.is_empty() && !attributes.starts_with(char::is_whitespace))
                    || attributes.to_ascii_lowercase().contains("span")
                {
                    return None;
                }
                Some(text)
            })
            .collect()
    }
    fn text(cell: &str) -> Option<String> {
        let value = cell_text(cell)?;
        let remaining = &cell[value.len()..];
        // Only a plain value or one complete link is supported.
        let remaining = if cell.starts_with("<a") {
            let (_, content) = cell.split_once('>')?;
            content.strip_prefix(value)?.strip_prefix("</a>")?
        } else {
            remaining
        };
        if !remaining.trim().is_empty()
            && remaining.trim() != "</td>"
            && remaining.trim() != "</th>"
        {
            return None;
        }
        html_unescape(value.trim())
    }
    let mut found = None;
    let mut remaining = page;
    while let Some((_, table)) = remaining.split_once("<table") {
        let (attributes, table) = table.split_once('>')?;
        if !attributes.is_empty() && !attributes.starts_with(char::is_whitespace) {
            return None;
        }
        let (table, rest) = table.split_once("</table>")?;
        if table.contains("<table") {
            return None;
        }
        remaining = rest;
        let mut header = false;
        for row in table.split("<tr").skip(1) {
            let (attributes, row) = row.split_once('>')?;
            if (!attributes.is_empty() && !attributes.starts_with(char::is_whitespace))
                || attributes.to_ascii_lowercase().contains("span")
            {
                return None;
            }
            let row = row
                .trim()
                .strip_suffix("</tr>")
                .unwrap_or(row.trim())
                .trim();
            let headings = cells(row, "th")?;
            let values = cells(row, "td")?;
            if !headings.is_empty() {
                if !values.is_empty() || header {
                    return None;
                }
                let mut decoded = headings
                    .iter()
                    .map(|cell| text(cell))
                    .collect::<Option<Vec<_>>>()?;
                if decoded.get(9).map(String::as_str) == Some("Motto (Ctrl-M)") {
                    decoded[9] = "Motto".to_owned();
                }
                if decoded.get(10).map(String::as_str) == Some("Guild (Ctrl-G)") {
                    decoded[10] = "Guild".to_owned();
                }
                header = decoded == HEADERS;
                continue;
            }
            let decoded = values
                .iter()
                .map(|cell| text(cell))
                .collect::<Option<Vec<_>>>()?;
            if decoded.get(1).map(String::as_str) != Some(name) {
                continue;
            }
            if !header || !(10..=11).contains(&decoded.len()) || found.is_some() {
                return None;
            }
            let guild = if decoded.len() == 10 || decoded[10].is_empty() {
                if values.get(10).is_some_and(|cell| cell.contains("<a")) {
                    return None;
                }
                None
            } else if values[10].starts_with("<a") && values[10].contains("guilds.php?id=") {
                Some(decoded[10].clone())
            } else {
                return None;
            };
            found = Some((decoded[1..10].to_vec(), guild));
        }
    }
    found
}

pub(crate) fn cell_text(cell: &str) -> Option<&str> {
    let text = if let Some(anchor) = cell.strip_prefix("<a") {
        anchor.split_once('>')?.1
    } else {
        cell
    };
    Some(text.split('<').next().unwrap_or_default())
}

pub(crate) fn html_unescape(value: &str) -> Option<String> {
    let mut decoded = String::with_capacity(value.len());
    let mut remaining = value;
    while let Some(offset) = remaining.find('&') {
        decoded.push_str(&remaining[..offset]);
        remaining = &remaining[offset..];
        let (replacement, length) = [
            ("&amp;", "&"),
            ("&lt;", "<"),
            ("&gt;", ">"),
            ("&quot;", "\""),
            ("&#39;", "'"),
        ]
        .into_iter()
        .find_map(|(entity, replacement)| {
            remaining
                .starts_with(entity)
                .then_some((replacement, entity.len()))
        })?;
        decoded.push_str(replacement);
        remaining = &remaining[length..];
    }
    decoded.push_str(remaining);
    Some(decoded)
}

#[cfg(test)]
fn fingerprint_values<'a>(
    prior_guild: &'a str,
    submitted_guild: &'a str,
) -> DesktopGuildFingerprintValues<'a> {
    DesktopGuildFingerprintValues {
        character_name: "Character",
        account: "account",
        password: "password",
        authorization: "Basic token",
        passkey: "73",
        prior_guild,
        submitted_guild,
    }
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
        let values = fingerprint_values("Old Guild", "New Guild");
        let accepted = values.fingerprint(b"Accepted New Guild");
        let rejected = values.fingerprint(b"Rejected New Guild");
        DesktopGuildResponseRules::from_evidence(
            "Synthetic Realm",
            operation,
            &DesktopGuildEvidence {
                profile: CompatibilityProfile::Desktop644,
                realm: "Synthetic Realm",
                operation,
                normalization: DESKTOP_RESPONSE_FINGERPRINT_VERSION,
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
            normalization: DESKTOP_RESPONSE_FINGERPRINT_VERSION,
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
    fn pemptus_rules_require_public_confirmation_even_with_synthetic_fingerprints() {
        use crate::desktop_evidence::{
            DesktopGuildFingerprints, synthetic_pemptus_evidence, with_synthetic_desktop_evidence,
        };
        let values = fingerprint_values("Old Guild", "New Guild");
        let record = synthetic_pemptus_evidence(
            crate::desktop_eligibility::DesktopOnlineOperation::Guild,
            Some(DesktopGuildFingerprints {
                normalization: DESKTOP_RESPONSE_FINGERPRINT_VERSION.to_owned(),
                join: values.fingerprint(b"Joined New Guild"),
                change: Some(values.fingerprint(b"Changed New Guild")),
                leave: values.fingerprint(b"Left Old Guild"),
                rejected: values.fingerprint(b"Rejected New Guild"),
            }),
        );
        with_synthetic_desktop_evidence(vec![record], || {
            let rules = DesktopGuildResponseRules::production(
                "Pemptus",
                DesktopGuildOperation::JoinOrChange,
            )
            .unwrap();
            assert_eq!(
                rules.classify(200, b"Joined New Guild", &values),
                DesktopGuildOutcome::Indeterminate
            );
            assert_eq!(
                rules.classify(200, b"Changed New Guild", &values),
                DesktopGuildOutcome::Indeterminate
            );
            assert_eq!(
                rules.classify(200, b"Rejected New Guild", &values),
                DesktopGuildOutcome::Indeterminate
            );
            assert_eq!(
                rules.classify(200, b"Unknown", &values),
                DesktopGuildOutcome::Indeterminate
            );
            let leave =
                DesktopGuildResponseRules::production("Pemptus", DesktopGuildOperation::Leave)
                    .unwrap();
            assert_eq!(
                leave.classify(200, b"Left Old Guild", &values),
                DesktopGuildOutcome::Indeterminate
            );
            assert!(
                DesktopGuildResponseRules::production(
                    "Spoltog",
                    DesktopGuildOperation::JoinOrChange
                )
                .is_err()
            );
        });
        assert!(
            DesktopGuildResponseRules::production("Pemptus", DesktopGuildOperation::JoinOrChange)
                .is_ok()
        );
        for adaptations in [
            &[crate::compatibility::DesktopAdaptation::LegacyPrologue62][..],
            &[
                crate::compatibility::DesktopAdaptation::LegacyQuestPlaceholder,
                crate::compatibility::DesktopAdaptation::LoadSpellingPatch,
            ][..],
        ] {
            assert!(
                DesktopGuildResponseRules::production_for_import(
                    "Pemptus",
                    DesktopGuildOperation::JoinOrChange,
                    adaptations,
                )
                .is_err()
            );
        }
    }

    #[test]
    fn unknown_oversized_rejected_and_failed_guild_results_preserve_membership() {
        let rules = rules(DesktopGuildOperation::JoinOrChange);
        let values = fingerprint_values("Old Guild", "New Guild");
        let cases = [
            rules.classify(200, b"Rejected New Guild", &values),
            rules.classify(200, b"Unknown New Guild", &values),
            rules.classify(
                200,
                &vec![b'x'; MAX_DESKTOP_GUILD_RESPONSE_BYTES + 1],
                &values,
            ),
            rules.classify(503, b"Accepted New Guild", &values),
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
    fn evidence_generated_fingerprints_classify_through_production_rules() {
        let join_values = DesktopGuildFingerprintValues {
            character_name: "Guild",
            account: "account",
            password: "password",
            authorization: "Basic token",
            passkey: "73",
            prior_guild: "Guild Hall",
            submitted_guild: "New Guild",
        };
        let join_body =
            b"Accepted Guild for account password Basic token 73 from Guild Hall to New Guild";
        let rejected_body =
            b"Rejected Guild for account password Basic token 73 from Guild Hall to New Guild";
        let accepted = join_values.fingerprint(join_body);
        let rejected = join_values.fingerprint(rejected_body);
        let join_rules = DesktopGuildResponseRules::from_evidence(
            "Synthetic Realm",
            DesktopGuildOperation::JoinOrChange,
            &DesktopGuildEvidence {
                profile: CompatibilityProfile::Desktop644,
                realm: "Synthetic Realm",
                operation: DesktopGuildOperation::JoinOrChange,
                normalization: DESKTOP_RESPONSE_FINGERPRINT_VERSION,
                accepted_fingerprint: &accepted,
                rejected_fingerprint: &rejected,
            },
        )
        .unwrap();
        assert_eq!(
            join_rules.classify(200, join_body, &join_values),
            DesktopGuildOutcome::Accepted
        );
        assert_eq!(
            join_rules.classify(200, rejected_body, &join_values),
            DesktopGuildOutcome::Rejected
        );

        let leave_values = DesktopGuildFingerprintValues {
            submitted_guild: "",
            ..join_values
        };
        let leave_body = b"Accepted Guild leaving Guild Hall for account with 73";
        let leave_accepted = leave_values.fingerprint(leave_body);
        let leave_rules = DesktopGuildResponseRules::from_evidence(
            "Synthetic Realm",
            DesktopGuildOperation::Leave,
            &DesktopGuildEvidence {
                profile: CompatibilityProfile::Desktop644,
                realm: "Synthetic Realm",
                operation: DesktopGuildOperation::Leave,
                normalization: DESKTOP_RESPONSE_FINGERPRINT_VERSION,
                accepted_fingerprint: &leave_accepted,
                rejected_fingerprint: &rejected,
            },
        )
        .unwrap();
        assert_eq!(
            leave_rules.classify(200, leave_body, &leave_values),
            DesktopGuildOutcome::Accepted
        );
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
    fn shared_guild_matching_preserves_case_equivalence_and_empty_leave() {
        assert!(public_guild_matches(Some("BEERguild"), "BEERGuild"));
        assert!(public_guild_matches(Some("Guild B"), "gUiLd b"));
        assert!(public_guild_matches(None, ""));
        assert!(!public_guild_matches(None, "Guild"));
        assert!(!public_guild_matches(Some("Guild"), ""));
        assert!(!public_guild_matches(Some("Other"), "Guild"));
    }

    #[test]
    fn pemptus_requires_declared_unambiguous_guild_column() {
        let header = "<h1>Hall of Fame</h1><table><tr><th>Rank<th>Name<th>Race<th>Class<th>Level<th>Prime Stat<th>Plot Stage<th>Prized Item<th>Specialty<th>Motto<th>Guild";
        let row =
            "<tr><td>1<td>Hero<td>Race<td>Class<td>1<td>STR 1<td>Act I<td>Item<td>Skill<td>Motto";
        let omitted = format!("{header}{row}</table>");
        assert_eq!(
            public_guild_for_realm("Pemptus", &omitted, "Hero"),
            Some(None)
        );
        assert_eq!(public_guild_for_realm("Spoltog", &omitted, "Hero"), None);
        assert_eq!(public_guild_for_realm("Unknown", &omitted, "Hero"), None);
        let official = omitted
            .replace("<th>Motto", "<th>Motto (Ctrl-M)\n")
            .replace("<th>Guild", "<th>Guild (Ctrl-G)")
            .replace("<tr><td>1", "<tr class=bob><td align=right>1");
        assert_eq!(
            public_guild_for_realm("Pemptus", &official, "Hero"),
            Some(None)
        );
        for guild in ["Guild A", "Guild B"] {
            let page = format!("{header}{row}<td><a href='guilds.php?id=1'>{guild}</a></table>");
            assert_eq!(
                public_guild_for_realm("Pemptus", &page, "Hero"),
                Some(Some(guild.to_owned()))
            );
        }
        assert_eq!(
            public_guild_for_realm("Pemptus", &format!("{header}{row}<td></table>"), "Hero"),
            Some(None)
        );
        for page in [
            format!("<table>{row}</table>"),
            omitted.replace("<th>Guild", ""),
            omitted.replace("<th>Guild", "<th>Other"),
            omitted.replace("<td>Class", "<td colspan='2'>Class"),
            omitted.replace("<th>Name", "<th rowspan='2'>Name"),
            format!("{header}{row}<td><td>Extra</table>"),
            format!("{header}{row}{row}</table>"),
            omitted.replace("<td>Item", ""),
            omitted.replace("<td>Motto", "<td><b>Motto</b>"),
            format!("{header}{row}<td>Unlinked Guild</table>"),
            format!("{header}{row}<td><a href='guilds.php?id=1'></a></table>"),
        ] {
            assert_eq!(
                public_guild_for_realm("Pemptus", &page, "Hero"),
                None,
                "{page}"
            );
        }
    }

    #[test]
    fn public_guild_verification_reads_only_the_matching_character_guild_cell() {
        let page = concat!(
            "<table><tr><th>Rank<th>Name<th>Race<th>Class<th>Level<th>Prime Stat",
            "<th>Plot Stage<th>Prized Item<th>Specialty<th>Motto<th>Guild",
            "<tr><td>1<td>Other<td>Race<td>Class<td>1<td>STR 1<td>Act I",
            "<td>Item<td>Skill<td>Other motto<td>",
            "<tr><td>2<td>Kenjabob<td>Race<td>Class<td>4<td>CHA 17<td>Act I",
            "<td>Item<td>Skill<td>Kenjabob.<td><a href=\"guilds.php?id=7778#7778\">BEERguild</a>",
            "<tr><td>3<td>Next<td>Race<td>Class<td>1<td>STR 1<td>Act I",
            "<td>Item<td>Skill<td></table>"
        );

        assert_eq!(
            public_guild(page, "Kenjabob"),
            Some(Some("BEERguild".to_owned()))
        );
        assert_eq!(public_guild(page, "Other"), Some(None));
        assert_eq!(public_guild(page, "Missing"), None);
    }

    #[test]
    fn public_guild_verification_rejects_ambiguous_or_malformed_rows() {
        let duplicate = concat!(
            "<table><tr><td>1<td>Kenjabob<td>Race<td>Class<td>1<td>STR 1<td>Act I",
            "<td>Item<td>Skill<td>Motto<td><a href=\"guilds.php?id=1\">BEERguild</a>",
            "<tr><td>2<td>Kenjabob<td>Race<td>Class<td>1<td>STR 1<td>Act I",
            "<td>Item<td>Skill<td>Motto<td><a href=\"guilds.php?id=1\">BEERguild</a></table>"
        );
        let missing_guild_cell = concat!(
            "<table><tr><td>1<td>Kenjabob<td>Race<td>Class<td>1<td>STR 1<td>Act I",
            "<td>Item<td>Skill<td>Motto</table>"
        );
        let unlinked_guild = concat!(
            "<table><tr><td>1<td>Kenjabob<td>Race<td>Class<td>1<td>STR 1<td>Act I",
            "<td>Item<td>Skill<td>Motto<td>BEERguild</table>"
        );
        let unsupported_entity = concat!(
            "<table><tr><td>1<td>Kenjabob<td>Race<td>Class<td>1<td>STR 1<td>Act I",
            "<td>Item<td>Skill<td>Motto<td><a href=\"guilds.php?id=1\">BEER&copy;guild</a></table>"
        );

        for page in [
            duplicate,
            missing_guild_cell,
            unlinked_guild,
            unsupported_entity,
        ] {
            assert_eq!(public_guild(page, "Kenjabob"), None);
        }
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
