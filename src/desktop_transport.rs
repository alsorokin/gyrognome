use std::{fmt, io::Read, time::Duration};

use crate::desktop_eligibility::DesktopCredentialMode;
use base64::{Engine, engine::general_purpose::STANDARD};
use thiserror::Error;
use url::Url;

const MAX_RESPONSE_BYTES: u64 = 16 * 1024;
const MAX_PUBLIC_PROFILE_BYTES: u64 = 256 * 1024;

#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
pub enum DesktopTransportError {
    #[error("desktop endpoint has no verified HTTPS mapping")]
    UnverifiedEndpoint,
    #[error("desktop endpoint has no verified authentication contract")]
    MissingAuthenticationContract,
    #[error("verified desktop endpoint configuration is invalid")]
    InvalidVerifiedEndpoint,
    #[error("desktop transport rejected a redirect")]
    RedirectRejected,
    #[error("desktop transport rejected an HTTP downgrade")]
    HttpDowngrade,
    #[error("desktop transport rejected a credential-bearing redirect")]
    CredentialBearingRedirect,
    #[error("desktop request delivery failed")]
    DeliveryFailed,
    #[error("desktop credentials do not match the verified authentication contract")]
    InvalidCredentials,
}

#[derive(Clone, PartialEq, Eq)]
pub struct DesktopTransportCredentials {
    account: String,
    password: String,
}

impl DesktopTransportCredentials {
    pub fn new(account: impl Into<String>, password: impl Into<String>) -> Self {
        Self {
            account: account.into(),
            password: password.into(),
        }
    }

    pub(crate) fn authorization_header(
        &self,
        mode: DesktopCredentialMode,
    ) -> Result<Option<String>, DesktopTransportError> {
        validate_transport_credentials(mode, self)?;
        Ok(match mode {
            DesktopCredentialMode::AccountPassword => Some(format!(
                "Basic {}",
                STANDARD.encode(format!("{}:{}", self.account, self.password))
            )),
            DesktopCredentialMode::PasskeyOnly => None,
        })
    }
}

impl fmt::Debug for DesktopTransportCredentials {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DesktopTransportCredentials")
            .field("account", &"[redacted]")
            .field("password", &"[redacted]")
            .finish()
    }
}

#[derive(Clone)]
pub struct VerifiedDesktopEndpoint {
    realm: String,
    endpoint: Url,
    authentication: VerifiedAuthenticationContract,
}

impl VerifiedDesktopEndpoint {
    pub fn realm(&self) -> &str {
        &self.realm
    }
    pub fn credential_mode(&self) -> DesktopCredentialMode {
        self.authentication
    }
}

impl fmt::Debug for VerifiedDesktopEndpoint {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("VerifiedDesktopEndpoint")
            .field("endpoint", &"[verified-redacted]")
            .field("authentication", &self.authentication)
            .finish()
    }
}

type VerifiedAuthenticationContract = DesktopCredentialMode;

struct EndpointMapping<'a> {
    realm: &'a str,
    saved_endpoint: &'a str,
    verified_https_endpoint: &'a str,
    authentication: Option<VerifiedAuthenticationContract>,
}

pub fn resolve_verified_desktop_endpoint(
    realm: &str,
    saved_endpoint: &str,
) -> Result<VerifiedDesktopEndpoint, DesktopTransportError> {
    let contract = crate::desktop_contract::realm_contract(realm)
        .map_err(|_| DesktopTransportError::UnverifiedEndpoint)?;
    resolve_from_mappings(
        realm,
        saved_endpoint,
        &[EndpointMapping {
            realm: contract.realm,
            saved_endpoint: contract.saved_endpoint,
            verified_https_endpoint: contract.https_endpoint,
            authentication: Some(contract.credential_mode),
        }],
    )
}

fn resolve_from_mappings(
    realm: &str,
    saved_endpoint: &str,
    mappings: &[EndpointMapping<'_>],
) -> Result<VerifiedDesktopEndpoint, DesktopTransportError> {
    let mapping = mappings
        .iter()
        .find(|mapping| mapping.realm == realm && mapping.saved_endpoint == saved_endpoint)
        .ok_or(DesktopTransportError::UnverifiedEndpoint)?;
    let authentication = mapping
        .authentication
        .ok_or(DesktopTransportError::MissingAuthenticationContract)?;
    let endpoint = Url::parse(mapping.verified_https_endpoint)
        .map_err(|_| DesktopTransportError::InvalidVerifiedEndpoint)?;
    if endpoint.scheme() != "https"
        || endpoint.host_str().is_none()
        || endpoint.port().is_some()
        || !endpoint.username().is_empty()
        || endpoint.password().is_some()
        || endpoint.query().is_some()
        || endpoint.fragment().is_some()
    {
        return Err(DesktopTransportError::InvalidVerifiedEndpoint);
    }
    Ok(VerifiedDesktopEndpoint {
        realm: mapping.realm.to_owned(),
        endpoint,
        authentication,
    })
}

pub fn fetch_verified_desktop_public_profile(
    target: &VerifiedDesktopEndpoint,
    name: &str,
) -> Result<String, DesktopTransportError> {
    fetch_verified_desktop_public_profile_with(target, name, |endpoint| {
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .https_only(true)
            .max_redirects(0)
            .http_status_as_error(false)
            .timeout_global(Some(Duration::from_secs(30)))
            .build()
            .into();
        let mut response = agent
            .get(endpoint.as_str())
            .call()
            .map_err(|_| DesktopTransportError::DeliveryFailed)?;
        let status = response.status().as_u16();
        let redirect = if (300..400).contains(&status) {
            response
                .headers()
                .get("location")
                .and_then(|value| value.to_str().ok())
                .and_then(|value| Url::parse(value).ok())
        } else {
            None
        };
        let mut body = Vec::new();
        response
            .body_mut()
            .as_reader()
            .take(MAX_PUBLIC_PROFILE_BYTES + 1)
            .read_to_end(&mut body)
            .map_err(|_| DesktopTransportError::DeliveryFailed)?;
        Ok(DesktopHttpResponse {
            status,
            redirect,
            body,
        })
    })
}

fn fetch_verified_desktop_public_profile_with(
    target: &VerifiedDesktopEndpoint,
    name: &str,
    fetch_once: impl FnOnce(&Url) -> Result<DesktopHttpResponse, DesktopTransportError>,
) -> Result<String, DesktopTransportError> {
    let mut endpoint = target.endpoint.clone();
    endpoint.query_pairs_mut().append_pair("name", name);
    let response = fetch_once(&endpoint)?;
    if response.redirect.is_some() || !(200..300).contains(&response.status) {
        return Err(
            if response.redirect.is_some() || (300..400).contains(&response.status) {
                DesktopTransportError::RedirectRejected
            } else {
                DesktopTransportError::DeliveryFailed
            },
        );
    }
    if response.body.len() as u64 > MAX_PUBLIC_PROFILE_BYTES {
        return Err(DesktopTransportError::DeliveryFailed);
    }
    String::from_utf8(response.body).map_err(|_| DesktopTransportError::DeliveryFailed)
}

pub struct DesktopHttpRequest<'a> {
    pub endpoint: Url,
    pub credentials: &'a DesktopTransportCredentials,
    pub authentication: DesktopCredentialMode,
}

impl fmt::Debug for DesktopHttpRequest<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DesktopHttpRequest")
            .field("endpoint", &"[verified-redacted]")
            .field("credentials", &self.credentials)
            .finish()
    }
}

#[derive(Debug)]
pub struct DesktopHttpResponse {
    pub status: u16,
    pub redirect: Option<Url>,
    pub body: Vec<u8>,
}

pub trait DesktopHttpClient {
    fn send(
        &self,
        request: DesktopHttpRequest<'_>,
    ) -> Result<DesktopHttpResponse, DesktopTransportError>;
}

pub struct DesktopHttpsTransport<C> {
    client: C,
}

impl<C> DesktopHttpsTransport<C> {
    pub fn new(client: C) -> Self {
        Self { client }
    }
}

impl<C: DesktopHttpClient> DesktopHttpsTransport<C> {
    pub fn deliver(
        &self,
        target: &VerifiedDesktopEndpoint,
        encoded_query: &str,
        credentials: &DesktopTransportCredentials,
    ) -> Result<DesktopHttpResponse, DesktopTransportError> {
        validate_transport_credentials(target.authentication, credentials)?;
        let mut endpoint = target.endpoint.clone();
        endpoint.set_query(Some(encoded_query));
        let response = self.client.send(DesktopHttpRequest {
            endpoint,
            credentials,
            authentication: target.authentication,
        })?;
        if let Some(redirect) = response.redirect {
            if redirect.scheme() != "https" {
                return Err(DesktopTransportError::HttpDowngrade);
            }
            if !redirect.username().is_empty() || redirect.password().is_some() {
                return Err(DesktopTransportError::CredentialBearingRedirect);
            }
            return Err(DesktopTransportError::RedirectRejected);
        }
        if (300..400).contains(&response.status) {
            return Err(DesktopTransportError::RedirectRejected);
        }
        if response.body.len() as u64 > MAX_RESPONSE_BYTES {
            return Err(DesktopTransportError::DeliveryFailed);
        }
        Ok(response)
    }
}

pub struct UreqDesktopHttpClient {
    agent: ureq::Agent,
}

impl Default for UreqDesktopHttpClient {
    fn default() -> Self {
        Self {
            agent: ureq::Agent::config_builder()
                .https_only(true)
                .max_redirects(0)
                .http_status_as_error(false)
                .timeout_global(Some(Duration::from_secs(30)))
                .build()
                .into(),
        }
    }
}

impl DesktopHttpClient for UreqDesktopHttpClient {
    fn send(
        &self,
        request: DesktopHttpRequest<'_>,
    ) -> Result<DesktopHttpResponse, DesktopTransportError> {
        let mut response = self
            .prepare_request(&request)?
            .call()
            .map_err(|_| DesktopTransportError::DeliveryFailed)?;
        let status = response.status().as_u16();
        let redirect = if (300..400).contains(&status) {
            response
                .headers()
                .get("location")
                .and_then(|value| value.to_str().ok())
                .and_then(|value| Url::parse(value).ok())
        } else {
            None
        };
        let mut body = Vec::new();
        response
            .body_mut()
            .as_reader()
            .take(MAX_RESPONSE_BYTES + 1)
            .read_to_end(&mut body)
            .map_err(|_| DesktopTransportError::DeliveryFailed)?;
        if body.len() as u64 > MAX_RESPONSE_BYTES {
            return Err(DesktopTransportError::DeliveryFailed);
        }
        Ok(DesktopHttpResponse {
            status,
            redirect,
            body,
        })
    }
}

fn validate_transport_credentials(
    mode: DesktopCredentialMode,
    credentials: &DesktopTransportCredentials,
) -> Result<(), DesktopTransportError> {
    let valid = match mode {
        DesktopCredentialMode::AccountPassword => {
            !credentials.account.is_empty()
                && !credentials.password.is_empty()
                && credentials.account.is_ascii()
                && credentials.password.is_ascii()
        }
        DesktopCredentialMode::PasskeyOnly => {
            credentials.account.is_empty() && credentials.password.is_empty()
        }
    };
    if valid {
        Ok(())
    } else {
        Err(DesktopTransportError::InvalidCredentials)
    }
}

impl UreqDesktopHttpClient {
    fn prepare_request(
        &self,
        request: &DesktopHttpRequest<'_>,
    ) -> Result<ureq::RequestBuilder<ureq::typestate::WithoutBody>, DesktopTransportError> {
        let builder = self.agent.get(request.endpoint.as_str());
        Ok(
            match request
                .credentials
                .authorization_header(request.authentication)?
            {
                Some(authorization) => builder.header("Authorization", authorization),
                None => builder,
            },
        )
    }
}

pub fn deliver_verified_desktop_request(
    target: &VerifiedDesktopEndpoint,
    encoded_query: &str,
    credentials: &DesktopTransportCredentials,
) -> Result<DesktopHttpResponse, DesktopTransportError> {
    DesktopHttpsTransport::new(UreqDesktopHttpClient::default()).deliver(
        target,
        encoded_query,
        credentials,
    )
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use super::*;

    const VERIFIED: EndpointMapping<'static> = EndpointMapping {
        realm: "Synthetic Realm",
        saved_endpoint: "https://legacy.synthetic.invalid/",
        verified_https_endpoint: "https://verified.synthetic.invalid/report",
        authentication: Some(VerifiedAuthenticationContract::AccountPassword),
    };

    struct SyntheticClient {
        calls: RefCell<Vec<String>>,
        response: RefCell<Option<DesktopHttpResponse>>,
    }

    impl DesktopHttpClient for SyntheticClient {
        fn send(
            &self,
            request: DesktopHttpRequest<'_>,
        ) -> Result<DesktopHttpResponse, DesktopTransportError> {
            let debug = format!("{request:?}");
            assert!(!debug.contains("synthetic-account"));
            assert!(!debug.contains("synthetic-password"));
            self.calls
                .borrow_mut()
                .push(request.endpoint.origin().ascii_serialization());
            self.response
                .borrow_mut()
                .take()
                .ok_or(DesktopTransportError::DeliveryFailed)
        }
    }

    fn credentials() -> DesktopTransportCredentials {
        DesktopTransportCredentials::new("synthetic-account", "synthetic-password")
    }

    #[test]
    fn constructed_requests_use_only_the_selected_realms_authorization() {
        let client = UreqDesktopHttpClient::default();
        for contract in [
            crate::desktop_contract::SPOLTOG,
            crate::desktop_contract::PEMPTUS,
            crate::desktop_contract::SPOLTOG,
            crate::desktop_contract::PEMPTUS,
        ] {
            let credentials = match contract.credential_mode {
                DesktopCredentialMode::AccountPassword => credentials(),
                DesktopCredentialMode::PasskeyOnly => DesktopTransportCredentials::new("", ""),
            };
            let request = client
                .prepare_request(&DesktopHttpRequest {
                    endpoint: Url::parse(contract.https_endpoint).unwrap(),
                    credentials: &credentials,
                    authentication: contract.credential_mode,
                })
                .unwrap();
            let headers = request.headers_ref().unwrap();
            match contract.credential_mode {
                DesktopCredentialMode::PasskeyOnly => {
                    assert!(!headers.contains_key("Authorization"))
                }
                DesktopCredentialMode::AccountPassword => assert_eq!(
                    headers["Authorization"],
                    format!(
                        "Basic {}",
                        STANDARD.encode("synthetic-account:synthetic-password")
                    ),
                ),
            }
        }
    }

    #[test]
    fn wrong_credentials_fail_before_the_adapter_and_response_limits_apply_once() {
        for (contract, credentials) in [
            (crate::desktop_contract::PEMPTUS, credentials()),
            (
                crate::desktop_contract::PEMPTUS,
                DesktopTransportCredentials::new("a", ""),
            ),
            (
                crate::desktop_contract::SPOLTOG,
                DesktopTransportCredentials::new("", ""),
            ),
        ] {
            let target =
                resolve_verified_desktop_endpoint(contract.realm, contract.saved_endpoint).unwrap();
            let transport = DesktopHttpsTransport::new(SyntheticClient {
                calls: RefCell::new(vec![]),
                response: RefCell::new(None),
            });
            assert_eq!(
                transport
                    .deliver(&target, "cmd=b", &credentials)
                    .unwrap_err(),
                DesktopTransportError::InvalidCredentials
            );
            assert!(transport.client.calls.borrow().is_empty());
        }
        for response in [
            None,
            Some(DesktopHttpResponse {
                status: 200,
                redirect: None,
                body: vec![b'x'; MAX_RESPONSE_BYTES as usize + 1],
            }),
        ] {
            let transport = DesktopHttpsTransport::new(SyntheticClient {
                calls: RefCell::new(vec![]),
                response: RefCell::new(response),
            });
            let target = resolve_verified_desktop_endpoint(
                crate::desktop_contract::SPOLTOG.realm,
                crate::desktop_contract::SPOLTOG.saved_endpoint,
            )
            .unwrap();
            assert_eq!(
                transport
                    .deliver(&target, "cmd=b", &credentials())
                    .unwrap_err(),
                DesktopTransportError::DeliveryFailed
            );
            assert_eq!(transport.client.calls.borrow().len(), 1);
        }
    }

    #[test]
    fn production_mapping_accepts_only_the_evidenced_spoltog_endpoint() {
        assert!(
            resolve_verified_desktop_endpoint("Spoltog", "http://progressquest.com/spoltog.php?")
                .is_ok()
        );
        for (realm, endpoint) in [
            ("Spoltog", "http://progressquest.com/spoltog.php"),
            ("Spoltog", "https://progressquest.com/spoltog.php"),
            ("Other Realm", "http://progressquest.com/spoltog.php?"),
        ] {
            assert_eq!(
                resolve_verified_desktop_endpoint(realm, endpoint).unwrap_err(),
                DesktopTransportError::UnverifiedEndpoint
            );
        }
    }

    #[test]
    fn production_mapping_is_closed_and_rejects_arbitrary_saved_destinations() {
        for (realm, endpoint) in [
            ("Synthetic Realm", "https://legacy.synthetic.invalid/"),
            (
                "Synthetic Realm",
                "https://account:password@legacy.synthetic.invalid/",
            ),
            ("Synthetic Realm", "http://legacy.synthetic.invalid/"),
            ("Other Realm", "https://legacy.synthetic.invalid/"),
        ] {
            assert_eq!(
                resolve_verified_desktop_endpoint(realm, endpoint).unwrap_err(),
                DesktopTransportError::UnverifiedEndpoint
            );
        }
    }

    #[test]
    fn mapping_requires_an_explicit_authentication_contract_and_clean_https_target() {
        let no_authentication = EndpointMapping {
            authentication: None,
            ..VERIFIED
        };
        assert_eq!(
            resolve_from_mappings(
                no_authentication.realm,
                no_authentication.saved_endpoint,
                &[no_authentication],
            )
            .unwrap_err(),
            DesktopTransportError::MissingAuthenticationContract
        );

        for destination in [
            "http://verified.synthetic.invalid/report",
            "https://account:password@verified.synthetic.invalid/report",
            "https://verified.synthetic.invalid:8443/report",
            "https://verified.synthetic.invalid/report?existing=query",
        ] {
            let invalid = EndpointMapping {
                verified_https_endpoint: destination,
                ..VERIFIED
            };
            assert_eq!(
                resolve_from_mappings(invalid.realm, invalid.saved_endpoint, &[invalid])
                    .unwrap_err(),
                DesktopTransportError::InvalidVerifiedEndpoint
            );
        }
    }

    #[test]
    fn rejects_redirects_without_replaying_credentials() {
        let target =
            resolve_from_mappings(VERIFIED.realm, VERIFIED.saved_endpoint, &[VERIFIED]).unwrap();
        let cases = [
            (
                "http://verified.synthetic.invalid/report",
                DesktopTransportError::HttpDowngrade,
            ),
            (
                "https://account:password@verified.synthetic.invalid/report",
                DesktopTransportError::CredentialBearingRedirect,
            ),
            (
                "https://other.synthetic.invalid/report",
                DesktopTransportError::RedirectRejected,
            ),
        ];
        for (location, expected) in cases {
            let client = SyntheticClient {
                calls: RefCell::new(Vec::new()),
                response: RefCell::new(Some(DesktopHttpResponse {
                    status: 302,
                    redirect: Some(Url::parse(location).unwrap()),
                    body: Vec::new(),
                })),
            };
            let transport = DesktopHttpsTransport::new(client);
            assert_eq!(
                transport
                    .deliver(&target, "cmd=b&rev=8", &credentials())
                    .unwrap_err(),
                expected
            );
            assert_eq!(transport.client.calls.borrow().len(), 1);
        }
    }

    #[test]
    fn diagnostics_never_include_credentials_urls_or_queries() {
        let target =
            resolve_from_mappings(VERIFIED.realm, VERIFIED.saved_endpoint, &[VERIFIED]).unwrap();
        let credentials = credentials();
        let target_debug = format!("{target:?}");
        let credentials_debug = format!("{credentials:?}");
        let error = DesktopTransportError::UnverifiedEndpoint.to_string();
        for sensitive in [
            "synthetic-account",
            "synthetic-password",
            "legacy.synthetic.invalid",
            "verified.synthetic.invalid",
            "cmd=b",
        ] {
            assert!(!target_debug.contains(sensitive));
            assert!(!credentials_debug.contains(sensitive));
            assert!(!error.contains(sensitive));
        }
    }

    #[test]
    fn public_profile_fetch_is_credential_free_bounded_and_redirect_closed() {
        let target =
            resolve_from_mappings(VERIFIED.realm, VERIFIED.saved_endpoint, &[VERIFIED]).unwrap();
        let body = b"<table>public profile</table>".to_vec();
        assert_eq!(
            fetch_verified_desktop_public_profile_with(&target, "Kenja bob", |endpoint| {
                assert!(endpoint.username().is_empty());
                assert!(endpoint.password().is_none());
                assert_eq!(
                    endpoint
                        .query_pairs()
                        .find(|(key, _)| key == "name")
                        .unwrap()
                        .1,
                    "Kenja bob"
                );
                Ok(DesktopHttpResponse {
                    status: 200,
                    redirect: None,
                    body: body.clone(),
                })
            })
            .unwrap(),
            "<table>public profile</table>"
        );

        assert_eq!(
            fetch_verified_desktop_public_profile_with(&target, "Kenjabob", |_| {
                Ok(DesktopHttpResponse {
                    status: 200,
                    redirect: None,
                    body: vec![b'x'; MAX_PUBLIC_PROFILE_BYTES as usize + 1],
                })
            })
            .unwrap_err(),
            DesktopTransportError::DeliveryFailed
        );
        assert_eq!(
            fetch_verified_desktop_public_profile_with(&target, "Kenjabob", |_| {
                Ok(DesktopHttpResponse {
                    status: 302,
                    redirect: Some(Url::parse("https://other.synthetic.invalid/profile").unwrap()),
                    body: Vec::new(),
                })
            })
            .unwrap_err(),
            DesktopTransportError::RedirectRejected
        );
        assert_eq!(
            fetch_verified_desktop_public_profile_with(&target, "Kenjabob", |_| {
                Err(DesktopTransportError::DeliveryFailed)
            })
            .unwrap_err(),
            DesktopTransportError::DeliveryFailed
        );
    }
}
