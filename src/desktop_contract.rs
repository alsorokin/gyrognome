use crate::desktop_eligibility::{DesktopCredentialMode, DesktopIneligibilityReason};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DesktopRealmContract {
    pub realm: &'static str,
    pub saved_endpoint: &'static str,
    pub https_endpoint: &'static str,
    pub credential_mode: DesktopCredentialMode,
    pub implementation_identity: &'static str,
}

pub const SPOLTOG: DesktopRealmContract = DesktopRealmContract {
    realm: "Spoltog",
    saved_endpoint: "http://progressquest.com/spoltog.php?",
    https_endpoint: "https://progressquest.com/spoltog.php",
    credential_mode: DesktopCredentialMode::AccountPassword,
    implementation_identity: "desktop-online-contract/v1",
};

pub const PEMPTUS: DesktopRealmContract = DesktopRealmContract {
    realm: "Pemptus",
    saved_endpoint: "http://progressquest.com/pemptus.php?",
    https_endpoint: "https://progressquest.com/pemptus.php",
    credential_mode: DesktopCredentialMode::PasskeyOnly,
    implementation_identity: "desktop-online-contract/pemptus/v1",
};

pub fn realm_contract(realm: &str) -> Result<DesktopRealmContract, DesktopIneligibilityReason> {
    [SPOLTOG, PEMPTUS]
        .into_iter()
        .find(|contract| contract.realm == realm)
        .ok_or(DesktopIneligibilityReason::RealmMismatch)
}

impl DesktopRealmContract {
    pub fn validate(
        self,
        saved_endpoint: &str,
        passkey: i32,
        account: &str,
        password: &str,
        encoding_supported: bool,
    ) -> Result<(), DesktopIneligibilityReason> {
        if self.saved_endpoint != saved_endpoint {
            return Err(DesktopIneligibilityReason::EndpointMismatch);
        }
        if passkey <= 0 || account.is_empty() != password.is_empty() {
            return Err(DesktopIneligibilityReason::InvalidCredentials);
        }
        let mode = if account.is_empty() {
            DesktopCredentialMode::PasskeyOnly
        } else {
            DesktopCredentialMode::AccountPassword
        };
        if mode != self.credential_mode {
            return Err(DesktopIneligibilityReason::CredentialModeMismatch);
        }
        if !encoding_supported || !account.is_ascii() || !password.is_ascii() {
            return Err(DesktopIneligibilityReason::UnsupportedEncoding);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_contracts_reject_aliases_unknown_realms_and_wrong_credentials() {
        assert!(realm_contract("Unknown").is_err());
        assert!(realm_contract("pemptus").is_err());
        for contract in [SPOLTOG, PEMPTUS] {
            let (account, password) = match contract.credential_mode {
                DesktopCredentialMode::AccountPassword => ("synthetic", "password"),
                DesktopCredentialMode::PasskeyOnly => ("", ""),
            };
            assert!(
                contract
                    .validate(contract.saved_endpoint, 42, account, password, true)
                    .is_ok()
            );
            for endpoint in [
                contract.https_endpoint,
                "http://progressquest.com/pemptus.php",
                "http://account@progressquest.com/pemptus.php?",
                "https://other.invalid/",
            ] {
                assert!(
                    contract
                        .validate(endpoint, 42, account, password, true)
                        .is_err()
                );
            }
            for (key, login, secret, ascii) in [
                (0, account, password, true),
                (42, "only-login", "", true),
                (42, "", "only-password", true),
                (42, account, password, false),
                (42, "\u{e9}", "secret", true),
            ] {
                assert!(
                    contract
                        .validate(contract.saved_endpoint, key, login, secret, ascii)
                        .is_err()
                );
            }
        }
        assert!(
            PEMPTUS
                .validate(PEMPTUS.saved_endpoint, 42, "a", "b", true)
                .is_err()
        );
        assert!(
            SPOLTOG
                .validate(SPOLTOG.saved_endpoint, 42, "", "", true)
                .is_err()
        );
    }
}
