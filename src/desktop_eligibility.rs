use std::fmt;

use serde::{Deserialize, Serialize};

use crate::compatibility::{
    CompatibilityProfile, DesktopAdaptation, DesktopAdvancementProvenance, DesktopLayout,
    SourceFormat,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DesktopOnlineOperation {
    AutomaticLevel,
    AutomaticAct,
    ManualBrag,
    Motto,
    Guild,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DesktopCredentialMode {
    PasskeyOnly,
    AccountPassword,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DesktopIneligibilityReason {
    FreshOfficialClientImportRequired,
    ProfileMismatch,
    ImportPathMismatch,
    RealmMismatch,
    EndpointMismatch,
    InvalidCredentials,
    CredentialModeMismatch,
    UnsupportedEncoding,
    OperationEvidenceUnavailable,
}

impl fmt::Display for DesktopIneligibilityReason {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::FreshOfficialClientImportRequired => {
                "a fresh official-client import is required after local advancement"
            }
            Self::ProfileMismatch => "the compatibility profile is not covered",
            Self::ImportPathMismatch => "the desktop import path or adaptation set is not covered",
            Self::RealmMismatch => "the desktop realm is not covered",
            Self::EndpointMismatch => "the desktop endpoint is not covered",
            Self::InvalidCredentials => "the retained desktop credentials are invalid or missing",
            Self::CredentialModeMismatch => {
                "the retained desktop authentication mode is not covered"
            }
            Self::UnsupportedEncoding => {
                "the desktop request contains text outside the supported encoding"
            }
            Self::OperationEvidenceUnavailable => {
                "matching desktop operation evidence is unavailable"
            }
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", content = "reason", rename_all = "kebab-case")]
pub enum DesktopEligibilityDecision {
    Eligible,
    Ineligible(DesktopIneligibilityReason),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopOperationEligibility {
    pub operation: DesktopOnlineOperation,
    pub decision: DesktopEligibilityDecision,
}

pub(crate) struct DesktopEligibilityInput<'a> {
    pub profile: CompatibilityProfile,
    pub source_format: SourceFormat,
    pub layout: DesktopLayout,
    pub adaptations: &'a [DesktopAdaptation],
    pub advancement: DesktopAdvancementProvenance,
    pub passkey: i32,
    pub realm: &'a str,
    pub endpoint: &'a str,
    pub account: &'a str,
    pub password: &'a str,
    pub encoding_supported: bool,
    pub operation: DesktopOnlineOperation,
}

pub(crate) struct DesktopEligibilityEvidence<'a> {
    pub profile: CompatibilityProfile,
    pub source_format: SourceFormat,
    pub layout: DesktopLayout,
    pub adaptations: &'a [DesktopAdaptation],
    pub realm: &'a str,
    pub saved_endpoint: &'a str,
    pub credential_mode: DesktopCredentialMode,
    pub operations: &'a [DesktopOnlineOperation],
}

pub(crate) fn production_desktop_evidence() -> Option<DesktopEligibilityEvidence<'static>> {
    static ADAPTATIONS: [DesktopAdaptation; 0] = [];
    static OPERATIONS: [DesktopOnlineOperation; 5] = [
        DesktopOnlineOperation::AutomaticLevel,
        DesktopOnlineOperation::AutomaticAct,
        DesktopOnlineOperation::ManualBrag,
        DesktopOnlineOperation::Motto,
        DesktopOnlineOperation::Guild,
    ];
    crate::desktop_evidence::production_desktop_evidence_is_valid().then_some(
        DesktopEligibilityEvidence {
            profile: CompatibilityProfile::Desktop644,
            source_format: SourceFormat::DesktopDelphiComponentStream,
            layout: DesktopLayout::SupportedComponentStream,
            adaptations: &ADAPTATIONS,
            realm: "Spoltog",
            saved_endpoint: "http://progressquest.com/spoltog.php?",
            credential_mode: DesktopCredentialMode::AccountPassword,
            operations: &OPERATIONS,
        },
    )
}

pub(crate) fn evaluate_desktop_eligibility(
    input: &DesktopEligibilityInput<'_>,
    evidence: Option<&DesktopEligibilityEvidence<'_>>,
) -> DesktopEligibilityDecision {
    use DesktopEligibilityDecision::{Eligible, Ineligible};
    use DesktopIneligibilityReason::{
        CredentialModeMismatch, EndpointMismatch, FreshOfficialClientImportRequired,
        ImportPathMismatch, InvalidCredentials, OperationEvidenceUnavailable, ProfileMismatch,
        RealmMismatch, UnsupportedEncoding,
    };

    if input.advancement == DesktopAdvancementProvenance::LocalOnly {
        return Ineligible(FreshOfficialClientImportRequired);
    }
    if input.profile != CompatibilityProfile::Desktop644 {
        return Ineligible(ProfileMismatch);
    }
    if input.passkey <= 0
        || input.realm.is_empty()
        || input.endpoint.is_empty()
        || input.account.is_empty() != input.password.is_empty()
    {
        return Ineligible(InvalidCredentials);
    }
    if !input.encoding_supported {
        return Ineligible(UnsupportedEncoding);
    }

    let Some(evidence) = evidence else {
        return Ineligible(OperationEvidenceUnavailable);
    };
    if evidence.profile != input.profile {
        return Ineligible(ProfileMismatch);
    }
    if evidence.source_format != input.source_format
        || evidence.layout != input.layout
        || !adaptations_match_evidence(input.adaptations, evidence.adaptations)
    {
        return Ineligible(ImportPathMismatch);
    }
    if evidence.realm != input.realm {
        return Ineligible(RealmMismatch);
    }
    if evidence.saved_endpoint != input.endpoint {
        return Ineligible(EndpointMismatch);
    }
    let credential_mode = if input.account.is_empty() {
        DesktopCredentialMode::PasskeyOnly
    } else {
        DesktopCredentialMode::AccountPassword
    };
    if evidence.credential_mode != credential_mode {
        return Ineligible(CredentialModeMismatch);
    }
    if !evidence.operations.contains(&input.operation) {
        return Ineligible(OperationEvidenceUnavailable);
    }
    Eligible
}

fn adaptations_match_evidence(input: &[DesktopAdaptation], evidence: &[DesktopAdaptation]) -> bool {
    input == evidence || (evidence.is_empty() && input == [DesktopAdaptation::LoadSpellingPatch])
}

#[cfg(test)]
mod tests {
    use super::*;

    const ADAPTATIONS: [DesktopAdaptation; 1] = [DesktopAdaptation::LoadSpellingPatch];
    const OPERATIONS: [DesktopOnlineOperation; 1] = [DesktopOnlineOperation::ManualBrag];

    fn input() -> DesktopEligibilityInput<'static> {
        DesktopEligibilityInput {
            profile: CompatibilityProfile::Desktop644,
            source_format: SourceFormat::DesktopDelphiComponentStream,
            layout: DesktopLayout::SupportedComponentStream,
            adaptations: &ADAPTATIONS,
            advancement: DesktopAdvancementProvenance::Unadvanced,
            passkey: 42,
            realm: "Synthetic Realm",
            endpoint: "https://synthetic.invalid/",
            account: "account",
            password: "password",
            encoding_supported: true,
            operation: DesktopOnlineOperation::ManualBrag,
        }
    }

    fn evidence() -> DesktopEligibilityEvidence<'static> {
        DesktopEligibilityEvidence {
            profile: CompatibilityProfile::Desktop644,
            source_format: SourceFormat::DesktopDelphiComponentStream,
            layout: DesktopLayout::SupportedComponentStream,
            adaptations: &ADAPTATIONS,
            realm: "Synthetic Realm",
            saved_endpoint: "https://synthetic.invalid/",
            credential_mode: DesktopCredentialMode::AccountPassword,
            operations: &OPERATIONS,
        }
    }

    #[test]
    fn matching_scope_is_eligible_and_production_is_exactly_scoped() {
        assert_eq!(
            evaluate_desktop_eligibility(&input(), Some(&evidence())),
            DesktopEligibilityDecision::Eligible
        );
        let mut production_input = input();
        production_input.realm = "Spoltog";
        production_input.endpoint = "http://progressquest.com/spoltog.php?";
        production_input.operation = DesktopOnlineOperation::AutomaticLevel;
        production_input.adaptations = &[];
        assert_eq!(
            evaluate_desktop_eligibility(&production_input, production_desktop_evidence().as_ref()),
            DesktopEligibilityDecision::Eligible
        );
    }

    #[test]
    fn production_evidence_covers_only_unadapted_and_spelling_normalized_imports() {
        let evidence = production_desktop_evidence().unwrap();
        for operation in [
            DesktopOnlineOperation::AutomaticLevel,
            DesktopOnlineOperation::AutomaticAct,
            DesktopOnlineOperation::ManualBrag,
            DesktopOnlineOperation::Motto,
            DesktopOnlineOperation::Guild,
        ] {
            for adaptations in [&[][..], &[DesktopAdaptation::LoadSpellingPatch][..]] {
                let mut value = input();
                value.realm = "Spoltog";
                value.endpoint = "http://progressquest.com/spoltog.php?";
                value.operation = operation;
                value.adaptations = adaptations;
                assert_eq!(
                    evaluate_desktop_eligibility(&value, Some(&evidence)),
                    DesktopEligibilityDecision::Eligible
                );
            }
        }

        for adaptations in [
            &[DesktopAdaptation::LegacyPrologue62][..],
            &[DesktopAdaptation::LegacyQuestPlaceholder][..],
            &[
                DesktopAdaptation::LoadSpellingPatch,
                DesktopAdaptation::LegacyPrologue62,
            ][..],
            &[
                DesktopAdaptation::LoadSpellingPatch,
                DesktopAdaptation::LegacyQuestPlaceholder,
            ][..],
        ] {
            let mut value = input();
            value.realm = "Spoltog";
            value.endpoint = "http://progressquest.com/spoltog.php?";
            value.adaptations = adaptations;
            assert_eq!(
                evaluate_desktop_eligibility(&value, Some(&evidence)),
                DesktopEligibilityDecision::Ineligible(
                    DesktopIneligibilityReason::ImportPathMismatch
                )
            );
        }
        assert!(serde_json::from_value::<DesktopAdaptation>(serde_json::json!("unknown")).is_err());
    }

    #[test]
    fn categorizes_every_desktop_gate_mismatch_without_sensitive_values() {
        let cases = [
            (
                {
                    let mut value = input();
                    value.advancement = DesktopAdvancementProvenance::LocalOnly;
                    value
                },
                evidence(),
                DesktopIneligibilityReason::FreshOfficialClientImportRequired,
            ),
            (
                {
                    let mut value = input();
                    value.profile = CompatibilityProfile::Browser;
                    value
                },
                evidence(),
                DesktopIneligibilityReason::ProfileMismatch,
            ),
            (
                input(),
                DesktopEligibilityEvidence {
                    adaptations: &[DesktopAdaptation::LegacyPrologue62],
                    ..evidence()
                },
                DesktopIneligibilityReason::ImportPathMismatch,
            ),
            (
                input(),
                DesktopEligibilityEvidence {
                    realm: "Other Realm",
                    ..evidence()
                },
                DesktopIneligibilityReason::RealmMismatch,
            ),
            (
                input(),
                DesktopEligibilityEvidence {
                    saved_endpoint: "https://other.invalid/",
                    ..evidence()
                },
                DesktopIneligibilityReason::EndpointMismatch,
            ),
            (
                {
                    let mut value = input();
                    value.passkey = 0;
                    value
                },
                evidence(),
                DesktopIneligibilityReason::InvalidCredentials,
            ),
            (
                input(),
                DesktopEligibilityEvidence {
                    credential_mode: DesktopCredentialMode::PasskeyOnly,
                    ..evidence()
                },
                DesktopIneligibilityReason::CredentialModeMismatch,
            ),
            (
                {
                    let mut value = input();
                    value.encoding_supported = false;
                    value
                },
                evidence(),
                DesktopIneligibilityReason::UnsupportedEncoding,
            ),
            (
                input(),
                DesktopEligibilityEvidence {
                    operations: &[],
                    ..evidence()
                },
                DesktopIneligibilityReason::OperationEvidenceUnavailable,
            ),
        ];

        for (input, evidence, expected) in cases {
            assert_eq!(
                evaluate_desktop_eligibility(&input, Some(&evidence)),
                DesktopEligibilityDecision::Ineligible(expected)
            );
            assert!(!expected.to_string().contains(input.realm));
            assert!(!expected.to_string().contains(input.endpoint));
            assert!(!expected.to_string().contains(input.account));
            assert!(!expected.to_string().contains(input.password));
        }
    }
}
