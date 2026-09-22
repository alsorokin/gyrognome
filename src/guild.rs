//! Bounded, credential-safe guild response interpretation.

use std::io::Read;

use ring::digest::{Context, SHA256};
use serde_json::Value;

use crate::fixtures::{GuildEvidenceError, validate_guild_evidence};

pub const MAX_RESPONSE_BYTES: usize = 16 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuildOutcome {
    Accepted,
    Rejected,
    Indeterminate,
}

impl GuildOutcome {
    pub fn message(self) -> &'static str {
        match self {
            Self::Accepted => "Guild designation accepted and saved.",
            Self::Rejected => "Guild designation rejected; previous guild retained.",
            Self::Indeterminate => "Guild outcome indeterminate; previous guild retained.",
        }
    }
}

pub struct GuildResponseRules {
    joined: String,
    rejected: String,
    left: String,
}

impl GuildResponseRules {
    pub fn bundled() -> Result<Self, GuildEvidenceError> {
        Self::from_evidence(include_str!(
            "../tests/fixtures/enrollment-conformance-evidence.json"
        ))
    }

    pub(crate) fn from_evidence(content: &str) -> Result<Self, GuildEvidenceError> {
        validate_guild_evidence(content)?;
        let evidence: Value =
            serde_json::from_str(content).map_err(|_| GuildEvidenceError::Malformed)?;
        let fingerprint = |key: &str| {
            evidence["guild"][key]["outcome"]["fingerprint"]
                .as_str()
                .map(str::to_owned)
                .ok_or(GuildEvidenceError::Incomplete)
        };
        Ok(Self {
            joined: fingerprint("nonEmpty")?,
            rejected: fingerprint("invalid")?,
            left: fingerprint("empty")?,
        })
    }

    pub fn read(&self, reader: impl Read, prior: &str, submitted: &str) -> GuildOutcome {
        let mut bytes = Vec::new();
        if reader
            .take((MAX_RESPONSE_BYTES + 1) as u64)
            .read_to_end(&mut bytes)
            .is_err()
            || bytes.len() > MAX_RESPONSE_BYTES
        {
            return GuildOutcome::Indeterminate;
        }

        let Ok(body) = std::str::from_utf8(&bytes) else {
            return GuildOutcome::Indeterminate;
        };
        self.classify_fingerprint(&fingerprint(body, prior, submitted), submitted.is_empty())
    }

    pub fn read_http(
        &self,
        status: u16,
        reader: impl Read,
        prior: &str,
        submitted: &str,
    ) -> GuildOutcome {
        if !(200..300).contains(&status) {
            return GuildOutcome::Indeterminate;
        }
        self.read(reader, prior, submitted)
    }

    fn classify_fingerprint(&self, fingerprint: &str, leaving: bool) -> GuildOutcome {
        if fingerprint == if leaving { &self.left } else { &self.joined } {
            GuildOutcome::Accepted
        } else if fingerprint == self.rejected {
            GuildOutcome::Rejected
        } else {
            GuildOutcome::Indeterminate
        }
    }
}

fn fingerprint(body: &str, prior: &str, submitted: &str) -> String {
    let mut designations = [prior, submitted];
    designations.sort_by_key(|value| std::cmp::Reverse(value.len()));
    let mut remaining = body;
    let mut digest = Context::new(&SHA256);
    while !remaining.is_empty() {
        if let Some(value) = designations
            .iter()
            .find(|value| !value.is_empty() && remaining.starts_with(**value))
        {
            digest.update(b"<guild-designation>");
            remaining = &remaining[value.len()..];
        } else {
            let length = remaining
                .chars()
                .next()
                .expect("nonempty response remainder")
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
    use super::*;
    use std::io::{self, Cursor};

    #[test]
    fn recognizes_each_live_fingerprint_only_for_its_operation() {
        let rules = GuildResponseRules::bundled().unwrap();
        assert_eq!(
            rules.classify_fingerprint(&rules.joined, false),
            GuildOutcome::Accepted
        );
        assert_eq!(
            rules.classify_fingerprint(&rules.left, true),
            GuildOutcome::Accepted
        );
        assert_eq!(
            rules.classify_fingerprint(&rules.rejected, false),
            GuildOutcome::Rejected
        );
        assert_eq!(
            rules.classify_fingerprint(&rules.left, false),
            GuildOutcome::Indeterminate
        );
        assert_eq!(
            rules.classify_fingerprint(&rules.joined, true),
            GuildOutcome::Indeterminate
        );
        assert_eq!(
            rules.classify_fingerprint("unknown", false),
            GuildOutcome::Indeterminate
        );
    }

    #[test]
    fn normalizes_single_pass_longest_first_unicode_and_literal_text() {
        assert_eq!(
            fingerprint("Old ABC, new AB.", "ABC", "AB"),
            "c26026a24d960802d64069b942fb3cc8403d4f2974a7e22a541f7804d73c0017"
        );
        assert_eq!(
            fingerprint("Joined Guild", "", "Guild"),
            "b199cc402211bdf782855ffcc840e2483ebf2269ce2a9282b5cf3055b6342689"
        );
        assert_eq!(
            fingerprint("Old ABC, new AB.", "ABC", "AB"),
            fingerprint("Old Longer, new Short.", "Longer", "Short")
        );
        assert_eq!(
            fingerprint("Joined A.* twice A.*", "", "A.*"),
            fingerprint("Joined Z twice Z", "", "Z")
        );
        assert_eq!(
            fingerprint("Joined \u{03b1}\u{03b2}", "", "\u{03b1}\u{03b2}"),
            fingerprint("Joined Guild", "", "Guild")
        );
        assert_eq!(
            fingerprint("Joined Guild", "Guild", "Guild"),
            fingerprint("Joined Guild", "", "Guild")
        );
        assert_eq!(
            fingerprint("", "", ""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn reader_is_bounded_and_failures_never_become_acceptance() {
        let body = "x".repeat(MAX_RESPONSE_BYTES);
        let rules = GuildResponseRules {
            joined: fingerprint(&body, "", ""),
            rejected: fingerprint("Rejected", "", ""),
            left: fingerprint("Left", "", ""),
        };
        assert_eq!(
            rules.read(body.as_bytes(), "", "Test"),
            GuildOutcome::Accepted
        );
        for status in [301, 400, 401, 403, 404, 500] {
            let mut reader = Cursor::new(body.as_bytes());
            assert_eq!(
                rules.read_http(status, &mut reader, "", "Test"),
                GuildOutcome::Indeterminate
            );
            assert_eq!(reader.position(), 0);
        }
        assert_eq!(
            rules.read(b"Rejected".as_slice(), "", "Test"),
            GuildOutcome::Rejected
        );
        assert_eq!(
            rules.read(b"Left".as_slice(), "Test", ""),
            GuildOutcome::Accepted
        );
        assert_eq!(
            rules.read(b"unknown".as_slice(), "", "Test"),
            GuildOutcome::Indeterminate
        );
        let mut oversized = Cursor::new(vec![b'x'; MAX_RESPONSE_BYTES * 2]);
        assert_eq!(
            rules.read(&mut oversized, "", "Test"),
            GuildOutcome::Indeterminate
        );
        assert_eq!(oversized.position(), (MAX_RESPONSE_BYTES + 1) as u64);
        assert_eq!(
            rules.read([0xff].as_slice(), "", ""),
            GuildOutcome::Indeterminate
        );
        struct Failing;
        impl Read for Failing {
            fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
                Err(io::Error::other("private failure"))
            }
        }
        assert_eq!(rules.read(Failing, "", ""), GuildOutcome::Indeterminate);
    }
}
