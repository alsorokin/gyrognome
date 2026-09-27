use std::{fmt, io::Read, time::Duration};

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
    endpoint: Url,
    authentication: VerifiedAuthenticationContract,
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

#[derive(Debug, Clone, Copy)]
struct VerifiedAuthenticationContract;

struct EndpointMapping<'a> {
    realm: &'a str,
    saved_endpoint: &'a str,
    verified_https_endpoint: &'a str,
    authentication: Option<VerifiedAuthenticationContract>,
}

const SPOLTOG_MAPPING: EndpointMapping<'static> = EndpointMapping {
    realm: "Spoltog",
    saved_endpoint: "http://progressquest.com/spoltog.php?",
    verified_https_endpoint: "https://progressquest.com/spoltog.php",
    authentication: Some(VerifiedAuthenticationContract),
};

pub fn resolve_verified_desktop_endpoint(
    realm: &str,
    saved_endpoint: &str,
) -> Result<VerifiedDesktopEndpoint, DesktopTransportError> {
    resolve_from_mappings(realm, saved_endpoint, &[SPOLTOG_MAPPING])
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
        let mut endpoint = target.endpoint.clone();
        endpoint.set_query(Some(encoded_query));
        let response = self.client.send(DesktopHttpRequest {
            endpoint,
            credentials,
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
        let authorization = format!(
            "Basic {}",
            STANDARD.encode(format!(
                "{}:{}",
                request.credentials.account, request.credentials.password
            ))
        );
        let mut response = self
            .agent
            .get(request.endpoint.as_str())
            .header("Authorization", &authorization)
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
        authentication: Some(VerifiedAuthenticationContract),
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
