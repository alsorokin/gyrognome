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

pub fn progress_report(
    host: &str,
    character: &Character,
    trigger: char,
    fields: ReportFields<'_>,
    passkey: i32,
) -> Result<String, ProtocolError> {
    let realm = character.online_realm.as_deref().unwrap_or_default();
    let query = format!(
        "cmd=b&t={trigger}&n={}&r={}&c={}&l={}&x={}&i={}&z={}&k={}&a={}&h={}&rev={REVISION}",
        url_encode(&character.name),
        url_encode(&character.race),
        url_encode(&character.class),
        character.level,
        fields.xp_position,
        url_encode(fields.best_equipment),
        url_encode(fields.best_spell),
        url_encode(fields.best_stat),
        url_encode(fields.best_plot),
        url_encode(realm)
    );
    let standardized = standardize_url(&format!("{host}{query}"))?;
    Ok(format!(
        "{standardized}&p={}&m={}",
        validator(&standardized, passkey),
        url_encode(fields.motto)
    ))
}

pub fn guild_request(
    host: &str,
    character: &Character,
    guild: &str,
    passkey: i32,
) -> Result<String, ProtocolError> {
    let realm = character.online_realm.as_deref().unwrap_or_default();
    let query = format!(
        "cmd=guild&n={}&r={}&c={}&l={}&h={}&rev={REVISION}&guild={}",
        url_encode(&character.name),
        url_encode(&character.race),
        url_encode(&character.class),
        character.level,
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
    use serde_json::json;

    use super::*;

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
        let character = Character {
            document: json!({}),
            name: "Test Hero".into(),
            race: "Gyrognome".into(),
            class: "Robot Monk".into(),
            level: 2,
            online_realm: Some("Alpaquil".into()),
        };
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
            "https://example.invalid/?cmd=b&t=l&n=Test+Hero&r=Gyrognome&c=Robot+Monk&l=2&x=1&"
        ));
        assert!(report.contains("&rev=6&p="));
    }

    #[test]
    fn constructs_creation_and_guild_requests() {
        let character = Character {
            document: json!({}),
            name: "Test Hero".into(),
            race: "Gyrognome".into(),
            class: "Robot Monk".into(),
            level: 2,
            online_realm: Some("Alpaquil".into()),
        };
        assert_eq!(
            create_request("https://example.invalid/?", "Test Hero", "Alpaquil"),
            "https://example.invalid/?cmd=create&name=Test+Hero&realm=Alpaquil&rev=6"
        );
        assert_eq!(
            guild_request("https://example.invalid/?", &character, "The Guild", 4242).unwrap(),
            "https://example.invalid/?cmd=guild&n=Test+Hero&r=Gyrognome&c=Robot+Monk&l=2&h=Alpaquil&rev=6&guild=The+Guild&p=-1638501661"
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
