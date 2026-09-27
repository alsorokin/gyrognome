use ring::digest::{SHA256, digest};

pub(crate) const DESKTOP_RESPONSE_FINGERPRINT_VERSION: &str =
    "desktop-response-redaction-sha256/v1";

pub(crate) struct DesktopGuildFingerprintValues<'a> {
    pub(crate) character_name: &'a str,
    pub(crate) account: &'a str,
    pub(crate) password: &'a str,
    pub(crate) authorization: &'a str,
    pub(crate) passkey: &'a str,
    pub(crate) prior_guild: &'a str,
    pub(crate) submitted_guild: &'a str,
}

impl DesktopGuildFingerprintValues<'_> {
    pub(crate) fn fingerprint(&self, body: &[u8]) -> String {
        normalized_response_fingerprint(
            body,
            [
                self.character_name,
                self.account,
                self.password,
                self.authorization,
                self.passkey,
                self.prior_guild,
                self.submitted_guild,
            ],
        )
    }
}

pub(crate) fn normalized_response_fingerprint<'a>(
    body: &[u8],
    dynamic_values: impl IntoIterator<Item = &'a str>,
) -> String {
    let text = String::from_utf8_lossy(body);
    let normalized = dynamic_values
        .into_iter()
        .filter(|value| !value.is_empty())
        .fold(text.into_owned(), |value, dynamic| {
            value.replace(dynamic, "<redacted>")
        });
    digest(&SHA256, normalized.as_bytes())
        .as_ref()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_the_live_normalization_algorithm() {
        assert_eq!(
            DESKTOP_RESPONSE_FINGERPRINT_VERSION,
            "desktop-response-redaction-sha256/v1"
        );
        assert_eq!(
            normalized_response_fingerprint(
                b"Accepted New Guild",
                [
                    "Character",
                    "account",
                    "password",
                    "Basic token",
                    "73",
                    "Old Guild",
                    "New Guild"
                ]
            ),
            "85ab65e30c405848bbc9a376ad7b57d8fe902907324bc7e5c2310ee7a49570b9"
        );
    }

    #[test]
    fn replacement_order_is_part_of_the_contract() {
        let ordered =
            normalized_response_fingerprint(b"Accepted Guild Guild Hall", ["Guild", "Guild Hall"]);
        let reversed =
            normalized_response_fingerprint(b"Accepted Guild Guild Hall", ["Guild Hall", "Guild"]);

        assert_ne!(ordered, reversed);
    }
}
