use serde::{Deserialize, Serialize};

use crate::simulation::ReportEvent;
use crate::state::Character;
use thiserror::Error;
use url::Url;

pub const REVISION: &str = "6";

#[derive(Debug, Error)]
pub enum ProtocolError {
    #[error("leaderboard endpoint is not a valid URL: {0}")]
    InvalidEndpoint(#[from] url::ParseError),
}

pub fn standardize_url(url: &str) -> Result<String, ProtocolError> {
    Ok(Url::parse(url)?.to_string())
}

pub fn create_request(host: &str, name: &str, realm: &str) -> String {
    format!(
        "{host}cmd=create&name={}&realm={realm}&rev={REVISION}",
        url_encode(name),
        realm = url_encode(realm)
    )
}

pub fn url_encode(value: &str) -> String {
    value
        .as_bytes()
        .iter()
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
            | b')' => vec![*byte as char],
            b' ' => vec!['+'],
            _ => format!("%{byte:02X}").chars().collect(),
        })
        .collect()
}

pub fn lfsr(plaintext: &str, salt: i32) -> i32 {
    let mut result = salt;
    for code_unit in plaintext.encode_utf16() {
        let feedback = 1 & ((result >> 31) ^ (result >> 5));
        result = result.wrapping_shl(1) ^ feedback ^ i32::from(code_unit);
    }
    for _ in 0..10 {
        let feedback = 1 & ((result >> 31) ^ (result >> 5));
        result = result.wrapping_shl(1) ^ feedback;
    }
    result
}

pub fn validator(query: &str, passkey: i32) -> i32 {
    let signed = query
        .split_once("cmd=")
        .map(|(_, tail)| format!("cmd={tail}"))
        .unwrap_or_else(|| query.to_owned());
    lfsr(&signed, passkey)
}

pub struct ReportFields<'a> {
    pub xp_position: u64,
    pub best_equipment: &'a str,
    pub best_spell: &'a str,
    pub best_stat: &'a str,
    pub best_plot: &'a str,
    pub motto: &'a str,
}

/// An unsigned progress-report parameter in browser field order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnsignedReportField {
    pub name: String,
    pub value: String,
}

/// Inspectable, transport-free progress-report construction output.
///
/// `normalized` is the standardized request prefix before its validator and
/// motto are appended. It is intentionally not a complete signed request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConstructedReport {
    pub unsigned_fields: Vec<UnsignedReportField>,
    pub normalized: String,
    pub validator: i32,
}

/// Constructs browser-ordered report data from a credential-free transition
/// event and a caller-supplied synthetic passkey.
pub fn progress_report_for_event(
    event: &ReportEvent,
    synthetic_passkey: i32,
) -> Result<ConstructedReport, ProtocolError> {
    let character = &event.snapshot.character;
    let host = character
        .online
        .as_ref()
        .map(|online| online.host.as_str())
        .unwrap_or_default();
    report_construction(
        host,
        event.trigger.code(),
        character,
        ReportFields {
            xp_position: character.progress.experience.position as u64,
            best_equipment: &character.bestequip,
            best_spell: &character.bestspell,
            best_stat: &character.beststat,
            best_plot: &character.plot.bestplot,
            motto: &event.motto,
        },
        synthetic_passkey,
    )
}

fn report_construction(
    host: &str,
    trigger: char,
    character: &Character,
    fields: ReportFields<'_>,
    passkey: i32,
) -> Result<ConstructedReport, ProtocolError> {
    let realm = character
        .online
        .as_ref()
        .map(|online| online.realm.as_str())
        .unwrap_or_default();
    let unsigned_fields = vec![
        ("cmd", "b".to_owned()),
        ("t", trigger.to_string()),
        ("n", character.traits.name.clone()),
        ("r", character.traits.race.clone()),
        ("c", character.traits.class.clone()),
        ("l", character.traits.level.to_string()),
        ("x", fields.xp_position.to_string()),
        ("i", fields.best_equipment.to_owned()),
        ("z", fields.best_spell.to_owned()),
        ("k", fields.best_stat.to_owned()),
        ("a", fields.best_plot.to_owned()),
        ("h", realm.to_owned()),
        ("rev", REVISION.to_owned()),
        ("m", fields.motto.to_owned()),
    ]
    .into_iter()
    .map(|(name, value)| UnsignedReportField {
        name: name.to_owned(),
        value,
    })
    .collect::<Vec<_>>();
    let query = unsigned_fields[..unsigned_fields.len() - 1]
        .iter()
        .map(|field| format!("{}={}", field.name, url_encode(&field.value)))
        .collect::<Vec<_>>()
        .join("&");
    let normalized = standardize_url(&format!("{host}{query}"))?;
    Ok(ConstructedReport {
        unsigned_fields,
        validator: validator(&normalized, passkey),
        normalized,
    })
}

pub fn progress_report(
    host: &str,
    character: &Character,
    trigger: char,
    fields: ReportFields<'_>,
    passkey: i32,
) -> Result<String, ProtocolError> {
    let motto = fields.motto.to_owned();
    let report = report_construction(host, trigger, character, fields, passkey)?;
    Ok(format!(
        "{}&p={}&m={}",
        report.normalized,
        report.validator,
        url_encode(&motto)
    ))
}

pub fn guild_request(
    host: &str,
    character: &Character,
    guild: &str,
    passkey: i32,
) -> Result<String, ProtocolError> {
    let realm = character
        .online
        .as_ref()
        .map(|online| online.realm.as_str())
        .unwrap_or_default();
    let query = format!(
        "cmd=guild&n={}&r={}&c={}&l={}&h={}&rev={REVISION}&guild={}",
        url_encode(&character.traits.name),
        url_encode(&character.traits.race),
        url_encode(&character.traits.class),
        character.traits.level,
        url_encode(realm),
        url_encode(guild)
    );
    let standardized = standardize_url(&format!("{host}{query}"))?;
    Ok(format!(
        "{standardized}&p={}",
        validator(&standardized, passkey)
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::{Engine, engine::general_purpose::STANDARD};

    fn character() -> Character {
        crate::save::import_text(
            &STANDARD.encode(include_str!("../tests/fixtures/reference-save.json")),
        )
        .unwrap()
    }

    #[test]
    fn encodes_like_browser_form_values() {
        assert_eq!(url_encode("A b&c/ü"), "A+b%26c%2F%C3%BC");
    }

    #[test]
    fn lfsr_is_signed_and_deterministic() {
        assert_eq!(lfsr("cmd=b&t=l", 4242), -2_052_713_391);
    }

    #[test]
    fn report_has_expected_order_and_signature() {
        let character = character();
        let report = progress_report(
            "https://example.invalid/?",
            &character,
            'l',
            ReportFields {
                xp_position: 1,
                best_equipment: "Rock",
                best_spell: "",
                best_stat: "INT 9",
                best_plot: "Act I",
                motto: "",
            },
            4242,
        )
        .unwrap();
        assert!(report.starts_with(
            "https://example.invalid/?cmd=b&t=l&n=Reference+Hero&r=Gyrognome&c=Robot+Monk&l=2&x=1&"
        ));
        assert!(report.contains("&rev=6&p="));
    }

    #[test]
    fn constructs_creation_and_guild_requests() {
        let character = character();
        assert_eq!(
            create_request("https://example.invalid/?", "Test Hero", "Alpaquil"),
            "https://example.invalid/?cmd=create&name=Test+Hero&realm=Alpaquil&rev=6"
        );
        assert_eq!(
            guild_request("https://example.invalid/?", &character, "The Guild", 4242).unwrap(),
            "https://example.invalid/?cmd=guild&n=Reference+Hero&r=Gyrognome&c=Robot+Monk&l=2&h=Alpaquil&rev=6&guild=The+Guild&p=-1681777756"
        );
    }

    #[test]
    fn standardizes_endpoints_before_signing() {
        assert_eq!(
            standardize_url("https://example.invalid").unwrap(),
            "https://example.invalid/"
        );
        assert!(standardize_url("not a URL").is_err());
    }
}
