use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    compatibility::DesktopCanonicalState,
    desktop_simulation::{
        DesktopProgressionError, DesktopReportSnapshot, DesktopReportTrigger,
        best_prime_stat_index, best_spell_index,
    },
};

pub const REVISION: &str = "8";
#[cfg(feature = "desktop-live-conformance")]
pub(crate) const PUBLIC_REPORT_FIELDS: [&str; 9] = ["n", "r", "c", "l", "k", "a", "i", "z", "m"];

#[derive(Debug, Error, PartialEq, Eq)]
pub enum DesktopProtocolError {
    #[error("desktop protocol state is invalid: {0}")]
    InvalidState(&'static str),
    #[error("desktop protocol value uses unsupported non-ASCII encoding: {0}")]
    UnsupportedEncoding(&'static str),
    #[error("desktop protocol passkey must be positive")]
    InvalidPasskey,
    #[error("desktop account authentication must contain both account and password")]
    InvalidAuthentication,
}

impl From<DesktopProgressionError> for DesktopProtocolError {
    fn from(_: DesktopProgressionError) -> Self {
        Self::InvalidState("report selection")
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct DesktopAccountAuthentication {
    account: String,
    password: String,
}

impl DesktopAccountAuthentication {
    pub fn new(account: impl Into<String>, password: impl Into<String>) -> Self {
        Self {
            account: account.into(),
            password: password.into(),
        }
    }

    fn validate(&self) -> Result<bool, DesktopProtocolError> {
        ensure_ascii(&self.account, "account")?;
        ensure_ascii(&self.password, "password")?;
        match (self.account.is_empty(), self.password.is_empty()) {
            (true, true) => Ok(false),
            (false, false) => Ok(true),
            _ => Err(DesktopProtocolError::InvalidAuthentication),
        }
    }
}

impl std::fmt::Debug for DesktopAccountAuthentication {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("DesktopAccountAuthentication")
            .field("account", &"[redacted]")
            .field("password", &"[redacted]")
            .finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DesktopReportOperation {
    Manual,
    Level,
    Act,
    Motto,
}

impl DesktopReportOperation {
    fn code(self) -> &'static str {
        match self {
            Self::Manual => "b",
            Self::Level => "l",
            Self::Act => "a",
            Self::Motto => "m",
        }
    }
}

impl From<DesktopReportTrigger> for DesktopReportOperation {
    fn from(trigger: DesktopReportTrigger) -> Self {
        match trigger {
            DesktopReportTrigger::Level => Self::Level,
            DesktopReportTrigger::Act => Self::Act,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DesktopProtocolField {
    pub name: String,
    pub value: String,
    pub encoded: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopAuthenticationSummary {
    pub credentials_present: bool,
    pub representation: String,
    pub retained_separately: bool,
    pub preview: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConstructedDesktopReport {
    pub fields_before_validator: Vec<DesktopProtocolField>,
    pub query_before_validator: String,
    pub synthetic_validator: i32,
    pub fields_after_validator: Vec<DesktopProtocolField>,
    pub authentication: DesktopAuthenticationSummary,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConstructedDesktopGuildRequest {
    pub fields_before_validator: Vec<DesktopProtocolField>,
    pub query_before_validator: String,
    pub synthetic_validator: i32,
    pub authentication: DesktopAuthenticationSummary,
}

impl ConstructedDesktopReport {
    #[cfg(feature = "desktop-live-conformance")]
    pub(crate) fn public_cells(&self) -> Option<Vec<String>> {
        PUBLIC_REPORT_FIELDS
            .into_iter()
            .map(|name| {
                self.fields_before_validator
                    .iter()
                    .chain(&self.fields_after_validator)
                    .find(|field| field.name == name)
                    .map(|field| field.value.clone())
                    .or_else(|| (name == "z").then(String::new))
            })
            .collect()
    }

    pub fn encoded_query(&self) -> String {
        let mut fields = self.fields_before_validator.clone();
        fields.push(DesktopProtocolField {
            name: "p".to_owned(),
            value: self.synthetic_validator.to_string(),
            encoded: self.synthetic_validator.to_string(),
        });
        fields.extend(self.fields_after_validator.clone());
        query(&fields)
    }
}

impl ConstructedDesktopGuildRequest {
    pub fn encoded_query(&self) -> String {
        let mut fields = self.fields_before_validator.clone();
        fields.push(DesktopProtocolField {
            name: "p".to_owned(),
            value: self.synthetic_validator.to_string(),
            encoded: self.synthetic_validator.to_string(),
        });
        query(&fields)
    }
}

pub fn report_for_snapshot(
    snapshot: &DesktopReportSnapshot,
    realm: &str,
    motto: &str,
    passkey: i32,
    authentication: &DesktopAccountAuthentication,
) -> Result<ConstructedDesktopReport, DesktopProtocolError> {
    report(
        &snapshot.state,
        snapshot.trigger.into(),
        realm,
        motto,
        passkey,
        authentication,
    )
}

pub fn report(
    state: &DesktopCanonicalState,
    operation: DesktopReportOperation,
    realm: &str,
    motto: &str,
    passkey: i32,
    authentication: &DesktopAccountAuthentication,
) -> Result<ConstructedDesktopReport, DesktopProtocolError> {
    validate_passkey(passkey)?;
    let credentials_present = authentication.validate()?;
    let mut fields = vec![field("cmd", "b")?, field("t", operation.code())?];
    fields.extend(trait_fields(state)?);
    fields.push(field("x", &state.bars.experience.position.to_string())?);
    fields.push(equipment_field(state)?);
    if let Some(spell) = spell_field(state)? {
        fields.push(spell);
    }
    fields.push(stat_field(state)?);
    fields.push(field(
        "a",
        &state
            .plots
            .last()
            .ok_or(DesktopProtocolError::InvalidState("plot history"))?
            .caption,
    )?);
    fields.push(field("h", realm)?);
    fields.push(field("rev", REVISION)?);
    let query_before_validator = query(&fields);
    let synthetic_validator = desktop_lfsr(&query_before_validator, passkey)?;
    Ok(ConstructedDesktopReport {
        fields_before_validator: fields,
        query_before_validator,
        synthetic_validator,
        fields_after_validator: vec![field("m", motto)?],
        authentication: authentication_summary(credentials_present),
    })
}

pub fn guild_request(
    state: &DesktopCanonicalState,
    realm: &str,
    guild: &str,
    passkey: i32,
    authentication: &DesktopAccountAuthentication,
) -> Result<ConstructedDesktopGuildRequest, DesktopProtocolError> {
    validate_passkey(passkey)?;
    let credentials_present = authentication.validate()?;
    let mut fields = vec![field("cmd", "guild")?];
    fields.extend(trait_fields(state)?);
    fields.push(field("h", realm)?);
    fields.push(field("rev", REVISION)?);
    fields.push(field("guild", guild)?);
    let query_before_validator = query(&fields);
    let synthetic_validator = desktop_lfsr(&query_before_validator, passkey)?;
    Ok(ConstructedDesktopGuildRequest {
        fields_before_validator: fields,
        query_before_validator,
        synthetic_validator,
        authentication: authentication_summary(credentials_present),
    })
}

pub fn desktop_form_encode(
    value: &str,
    field_name: &'static str,
) -> Result<String, DesktopProtocolError> {
    ensure_ascii(value, field_name)?;
    Ok(value
        .bytes()
        .flat_map(|byte| match byte {
            b'A'..=b'Z'
            | b'a'..=b'z'
            | b'0'..=b'9'
            | b'-'
            | b'_'
            | b'.'
            | b'!'
            | b'~'
            | b'*'
            | b'\''
            | b'('
            | b')' => vec![byte as char],
            b' ' => vec!['+'],
            _ => format!("%{byte:02X}").chars().collect(),
        })
        .collect())
}

pub fn desktop_lfsr(plaintext: &str, salt: i32) -> Result<i32, DesktopProtocolError> {
    ensure_ascii(plaintext, "validator input")?;
    let mut result = salt;
    for byte in plaintext.bytes() {
        let feedback = 1 & ((result >> 31) ^ (result >> 5));
        result = result.wrapping_shl(1) ^ feedback ^ i32::from(byte);
    }
    for _ in 0..10 {
        let feedback = 1 & ((result >> 31) ^ (result >> 5));
        result = result.wrapping_shl(1) ^ feedback;
    }
    Ok(result)
}

fn validate_passkey(passkey: i32) -> Result<(), DesktopProtocolError> {
    if passkey <= 0 {
        return Err(DesktopProtocolError::InvalidPasskey);
    }
    Ok(())
}

fn trait_fields(
    state: &DesktopCanonicalState,
) -> Result<Vec<DesktopProtocolField>, DesktopProtocolError> {
    if state.traits.is_empty() {
        return Err(DesktopProtocolError::InvalidState("traits"));
    }
    state
        .traits
        .iter()
        .map(|row| {
            let name = row
                .caption
                .bytes()
                .next()
                .filter(u8::is_ascii_alphabetic)
                .ok_or(DesktopProtocolError::InvalidState("trait caption"))?
                .to_ascii_lowercase();
            let name = char::from(name).to_string();
            let value = single_value(row, "trait value")?;
            field(&name, value)
        })
        .collect()
}

fn equipment_field(
    state: &DesktopCanonicalState,
) -> Result<DesktopProtocolField, DesktopProtocolError> {
    let equipment = state
        .equipment
        .get(state.prized_equipment)
        .ok_or(DesktopProtocolError::InvalidState("prized equipment"))?;
    let value = single_value(equipment, "equipment value")?;
    let value = if state.prized_equipment > 1 {
        format!("{value} {}", equipment.caption)
    } else {
        value.to_owned()
    };
    field("i", &value)
}

fn spell_field(
    state: &DesktopCanonicalState,
) -> Result<Option<DesktopProtocolField>, DesktopProtocolError> {
    let Some(index) = best_spell_index(&state.spells)? else {
        return Ok(None);
    };
    let spell = state
        .spells
        .get(index)
        .ok_or(DesktopProtocolError::InvalidState("spell index"))?;
    Ok(Some(field(
        "z",
        &format!("{} {}", spell.caption, single_value(spell, "spell rank")?),
    )?))
}

fn stat_field(state: &DesktopCanonicalState) -> Result<DesktopProtocolField, DesktopProtocolError> {
    let index = best_prime_stat_index(&state.stats)?;
    let stat = state
        .stats
        .get(index)
        .ok_or(DesktopProtocolError::InvalidState("prime stat index"))?;
    field(
        "k",
        &format!("{} {}", stat.caption, single_value(stat, "stat value")?),
    )
}

fn single_value<'a>(
    row: &'a crate::desktop_save::DesktopValidatedRow,
    field: &'static str,
) -> Result<&'a str, DesktopProtocolError> {
    if row.subitems.len() != 1 {
        return Err(DesktopProtocolError::InvalidState(field));
    }
    Ok(&row.subitems[0])
}

fn field(name: &str, value: &str) -> Result<DesktopProtocolField, DesktopProtocolError> {
    ensure_ascii(name, "field name")?;
    Ok(DesktopProtocolField {
        name: name.to_owned(),
        value: value.to_owned(),
        encoded: desktop_form_encode(value, "field value")?,
    })
}

fn query(fields: &[DesktopProtocolField]) -> String {
    fields
        .iter()
        .map(|field| format!("{}={}", field.name, field.encoded))
        .collect::<Vec<_>>()
        .join("&")
}

fn ensure_ascii(value: &str, field: &'static str) -> Result<(), DesktopProtocolError> {
    if !value.is_ascii() {
        return Err(DesktopProtocolError::UnsupportedEncoding(field));
    }
    Ok(())
}

fn authentication_summary(credentials_present: bool) -> DesktopAuthenticationSummary {
    DesktopAuthenticationSummary {
        credentials_present,
        representation: if credentials_present {
            "legacy-url-userinfo".to_owned()
        } else {
            "none".to_owned()
        },
        retained_separately: true,
        preview: "redacted".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::*;
    use crate::{
        desktop_save::{
            DesktopQuestMarker, DesktopValidatedBar, DesktopValidatedBars, DesktopValidatedProfile,
            DesktopValidatedRow,
        },
        desktop_simulation::DesktopReportTrigger,
    };

    fn row(caption: &str, value: &str) -> DesktopValidatedRow {
        DesktopValidatedRow {
            header: [0, -1, -1, 0, 1],
            caption: caption.to_owned(),
            subitems: vec![value.to_owned()],
        }
    }

    fn state() -> DesktopCanonicalState {
        DesktopCanonicalState {
            traits: vec![
                row("Name", "Synthetic Hero & Co."),
                row("Race", "Half Orc"),
                row("Class", "Robot Monk"),
                row("Level", "2"),
            ],
            stats: vec![
                row("STR", "12"),
                row("CON", "11"),
                row("DEX", "10"),
                row("INT", "9"),
                row("WIS", "8"),
                row("CHA", "7"),
            ],
            equipment: vec![
                row("Weapon", "Stick"),
                row("Shield", "Plate"),
                row("Shield", "Banded Buckler"),
            ],
            inventory: vec![row("Gold", "0")],
            spells: Vec::new(),
            plots: vec![row("Act I", "0")],
            quests: Vec::new(),
            current_task: "load".to_owned(),
            quest: DesktopQuestMarker::None,
            queue: Vec::new(),
            activity: "Loading...".to_owned(),
            bars: DesktopValidatedBars {
                experience: DesktopValidatedBar {
                    position: 42,
                    maximum: 100,
                },
                encumbrance: DesktopValidatedBar {
                    position: 0,
                    maximum: 22,
                },
                plot: DesktopValidatedBar {
                    position: 0,
                    maximum: 100,
                },
                quest: DesktopValidatedBar {
                    position: 0,
                    maximum: 100,
                },
                task: DesktopValidatedBar {
                    position: 0,
                    maximum: 2_000,
                },
            },
            prized_equipment: 2,
            game_style: 3,
            profile: DesktopValidatedProfile {
                motto: "Ready & waiting".to_owned(),
                guild: String::new(),
            },
        }
    }

    fn authentication() -> DesktopAccountAuthentication {
        DesktopAccountAuthentication::new("synthetic-account", "synthetic-password")
    }

    fn fixture() -> Value {
        serde_json::from_str(include_str!(
            "../tests/fixtures/desktop-protocol-vectors.json"
        ))
        .unwrap()
    }

    #[test]
    fn matches_source_derived_report_vectors_and_omits_empty_spell() {
        let expected = fixture();
        let base = state();
        let manual = report(
            &base,
            DesktopReportOperation::Manual,
            "Synthetic Realm",
            "Ready & waiting",
            42_424,
            &authentication(),
        )
        .unwrap();
        assert_eq!(
            serde_json::to_value(manual).unwrap(),
            expected["observations"]["reports"]["manual"]
        );

        let mut level = base.clone();
        level.bars.experience.position = 0;
        level.spells.push(row("Rabbit Punch", "II"));
        let level = report_for_snapshot(
            &DesktopReportSnapshot {
                trigger: DesktopReportTrigger::Level,
                state: level,
            },
            "Synthetic Realm",
            "Ready & waiting",
            42_424,
            &authentication(),
        )
        .unwrap();
        assert_eq!(
            serde_json::to_value(level).unwrap(),
            expected["observations"]["reports"]["level"]
        );

        assert!(
            expected["observations"]["reports"]["manual"]["fieldsBeforeValidator"]
                .as_array()
                .unwrap()
                .iter()
                .all(|field| field["name"] != "z")
        );
    }

    #[test]
    fn matches_source_derived_act_motto_and_guild_vectors() {
        let expected = fixture();
        let mut act = state();
        act.prized_equipment = 1;
        act.equipment[1].subitems[0] = "Polished Plate".to_owned();
        act.plots.push(row("Act II", "0"));
        let act = report(
            &act,
            DesktopReportOperation::Act,
            "Synthetic Realm",
            "Ready & waiting",
            42_424,
            &authentication(),
        )
        .unwrap();
        assert_eq!(
            serde_json::to_value(act).unwrap(),
            expected["observations"]["reports"]["act"]
        );

        let motto = report(
            &state(),
            DesktopReportOperation::Motto,
            "Synthetic Realm",
            "Changed motto!",
            42_424,
            &authentication(),
        )
        .unwrap();
        assert_eq!(
            serde_json::to_value(motto).unwrap(),
            expected["observations"]["reports"]["motto"]
        );

        let guild = guild_request(
            &state(),
            "Synthetic Realm",
            "The A&B Guild",
            42_424,
            &authentication(),
        )
        .unwrap();
        assert_eq!(
            serde_json::to_value(guild).unwrap(),
            expected["observations"]["guild"]
        );
    }

    #[test]
    fn rejects_unsupported_encoding_passkeys_and_partial_authentication() {
        let state = state();
        assert_eq!(
            report(
                &state,
                DesktopReportOperation::Manual,
                "Synthetic Realm",
                "Gnomé",
                42_424,
                &authentication(),
            )
            .unwrap_err(),
            DesktopProtocolError::UnsupportedEncoding("field value")
        );
        assert_eq!(
            report(
                &state,
                DesktopReportOperation::Manual,
                "Synthetic Realm",
                "",
                0,
                &authentication(),
            )
            .unwrap_err(),
            DesktopProtocolError::InvalidPasskey
        );
        assert_eq!(
            guild_request(
                &state,
                "Synthetic Realm",
                "Guild",
                42_424,
                &DesktopAccountAuthentication::new("account", ""),
            )
            .unwrap_err(),
            DesktopProtocolError::InvalidAuthentication
        );
    }

    #[test]
    fn authentication_debug_and_serialized_output_are_credential_safe() {
        let authentication = authentication();
        let output = report(
            &state(),
            DesktopReportOperation::Manual,
            "Synthetic Realm",
            "Ready & waiting",
            42_424,
            &authentication,
        )
        .unwrap();
        let debug = format!("{authentication:?}");
        let serialized = serde_json::to_string(&output).unwrap();
        for secret in ["synthetic-account", "synthetic-password"] {
            assert!(!debug.contains(secret));
            assert!(!serialized.contains(secret));
        }
        assert_eq!(
            serde_json::to_value(&output.authentication).unwrap(),
            json!({
                "credentialsPresent": true,
                "representation": "legacy-url-userinfo",
                "retainedSeparately": true,
                "preview": "redacted"
            })
        );
    }

    #[test]
    fn encoded_report_places_validator_before_motto() {
        let output = report(
            &state(),
            DesktopReportOperation::Manual,
            "Synthetic Realm",
            "Ready & waiting",
            42_424,
            &authentication(),
        )
        .unwrap();
        let query = output.encoded_query();
        let validator = query.find("&p=").unwrap();
        let motto = query.find("&m=").unwrap();
        assert!(validator < motto);
        assert_eq!(query.matches("&p=").count(), 1);
    }
}
