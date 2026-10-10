//! Credential-safe leaderboard report delivery.

use std::{process::Command, time::Duration};
use thiserror::Error;
use url::Url;

use crate::{
    compatibility::CompatibilityProfile,
    desktop_eligibility::DesktopOnlineOperation,
    desktop_fingerprint::DesktopGuildFingerprintValues,
    desktop_profile::{
        DesktopGuildOperation, DesktopGuildOutcome, DesktopGuildResponseRules,
        apply_desktop_guild_action, public_guild_for_realm, public_guild_matches,
    },
    desktop_protocol::{
        DesktopAccountAuthentication, DesktopReportOperation,
        guild_request as desktop_guild_request, report as desktop_report,
        report_for_snapshot as desktop_report_for_snapshot,
    },
    desktop_simulation::DesktopReportSnapshot,
    desktop_transport::{
        DesktopHttpResponse, DesktopTransportCredentials, DesktopTransportError,
        VerifiedDesktopEndpoint, deliver_verified_desktop_request,
        fetch_verified_desktop_public_profile, resolve_verified_desktop_endpoint,
    },
    fixtures::{EnrollmentEvidenceError, validate_bundled_enrollment_evidence},
    guild::{GuildOutcome, GuildResponseRules},
    protocol::{self, ReportFields},
    runtime::{CharacterId, CharacterIdentity, StorageError, Store},
    simulation::{ReportEvent, ReportTrigger},
    state::{Character, OnlineMetadata},
};

pub const OFFICIAL_LEADERBOARD_ENDPOINT: &str = "https://progressquest.com/alpaquil.php";
const OFFICIAL_LEADERBOARD_HOST: &str = "https://progressquest.com/alpaquil.php?";
const OFFICIAL_REALM: &str = "Alpaquil";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeliveryOutcome {
    Delivered,
    EndpointRejected,
    DeliveryFailed,
}

impl DeliveryOutcome {
    pub fn motto_message(self) -> &'static str {
        match self {
            Self::Delivered => "Motto saved. Report delivered; server acceptance is not confirmed.",
            Self::EndpointRejected => {
                "Motto saved. Report rejected by endpoint; saved motto retained."
            }
            Self::DeliveryFailed => "Motto saved. Report delivery failed; saved motto retained.",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CreateOutcome {
    Enrolled(i32),
    DuplicateName,
    Incomplete,
}

pub enum CreateDelivery {
    Response(String),
    DeliveryFailed,
}

#[derive(Debug, Error)]
pub enum ReportingError {
    #[error(transparent)]
    GuildEvidence(#[from] crate::fixtures::GuildEvidenceError),
    #[error(transparent)]
    Evidence(#[from] EnrollmentEvidenceError),
    #[error(transparent)]
    Storage(#[from] StorageError),
    #[error("managed character does not use the official leaderboard endpoint")]
    UnofficialEndpoint,
    #[error("could not construct the leaderboard report")]
    Construction,
    #[error("online profile text contains a control character")]
    InvalidProfileText,
    #[error("online enrollment is incomplete; no local character was registered")]
    IncompleteEnrollment,
    #[error(transparent)]
    DesktopTransport(#[from] DesktopTransportError),
    #[error("could not construct the desktop report")]
    DesktopConstruction,
    #[error(transparent)]
    DesktopProfile(#[from] crate::desktop_profile::DesktopProfileError),
}

pub trait ReportTransport: Send {
    fn deliver(&self, request: Url) -> DeliveryOutcome;

    fn deliver_desktop(
        &self,
        _target: &VerifiedDesktopEndpoint,
        _encoded_query: &str,
        _credentials: &DesktopTransportCredentials,
    ) -> Result<DesktopHttpResponse, DesktopTransportError> {
        Err(DesktopTransportError::DeliveryFailed)
    }
}

pub trait EnrollmentTransport: ReportTransport {
    fn create(&self, request: Url) -> CreateDelivery;
}

pub trait GuildTransport {
    fn guild(
        &self,
        request: Url,
        prior: &str,
        submitted: &str,
        rules: &GuildResponseRules,
    ) -> GuildOutcome;

    fn guild_desktop(
        &self,
        _target: &VerifiedDesktopEndpoint,
        _encoded_query: &str,
        _credentials: &DesktopTransportCredentials,
    ) -> Result<DesktopHttpResponse, DesktopTransportError> {
        Err(DesktopTransportError::DeliveryFailed)
    }

    fn desktop_public_guild(
        &self,
        _target: &VerifiedDesktopEndpoint,
        _name: &str,
    ) -> Result<Option<String>, DesktopTransportError> {
        Err(DesktopTransportError::DeliveryFailed)
    }
}

pub struct HttpsTransport;

impl GuildTransport for HttpsTransport {
    fn guild(
        &self,
        request: Url,
        prior: &str,
        submitted: &str,
        rules: &GuildResponseRules,
    ) -> GuildOutcome {
        if official_endpoint(request.as_str()).is_err() {
            return GuildOutcome::Indeterminate;
        }
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .https_only(true)
            .max_redirects(0)
            .http_status_as_error(false)
            .timeout_global(Some(Duration::from_secs(30)))
            .build()
            .into();
        match agent.get(request.as_str()).call() {
            Ok(mut response) => rules.read_http(
                response.status().as_u16(),
                response.body_mut().as_reader(),
                prior,
                submitted,
            ),
            Err(_) => GuildOutcome::Indeterminate,
        }
    }

    fn guild_desktop(
        &self,
        target: &VerifiedDesktopEndpoint,
        encoded_query: &str,
        credentials: &DesktopTransportCredentials,
    ) -> Result<DesktopHttpResponse, DesktopTransportError> {
        deliver_verified_desktop_request(target, encoded_query, credentials)
    }

    fn desktop_public_guild(
        &self,
        target: &VerifiedDesktopEndpoint,
        name: &str,
    ) -> Result<Option<String>, DesktopTransportError> {
        let page = fetch_verified_desktop_public_profile(target, name)?;
        public_guild_for_realm(target.realm(), &page, name)
            .ok_or(DesktopTransportError::DeliveryFailed)
    }
}

impl ReportTransport for HttpsTransport {
    fn deliver(&self, request: Url) -> DeliveryOutcome {
        match ureq::get(request.as_str()).call() {
            Ok(response) if response.status().is_success() => DeliveryOutcome::Delivered,
            Ok(_) => DeliveryOutcome::EndpointRejected,
            Err(_) => DeliveryOutcome::DeliveryFailed,
        }
    }

    fn deliver_desktop(
        &self,
        target: &VerifiedDesktopEndpoint,
        encoded_query: &str,
        credentials: &DesktopTransportCredentials,
    ) -> Result<DesktopHttpResponse, DesktopTransportError> {
        deliver_verified_desktop_request(target, encoded_query, credentials)
    }
}

impl EnrollmentTransport for HttpsTransport {
    fn create(&self, request: Url) -> CreateDelivery {
        match ureq::get(request.as_str()).call() {
            Ok(mut response) => response
                .body_mut()
                .read_to_string()
                .map(CreateDelivery::Response)
                .unwrap_or(CreateDelivery::DeliveryFailed),
            Err(_) => CreateDelivery::DeliveryFailed,
        }
    }
}

#[cfg(feature = "enrollment-test-transport")]
pub struct TestEnrollmentTransport {
    create: std::cell::RefCell<std::collections::VecDeque<CreateDelivery>>,
    report: DeliveryOutcome,
}

#[cfg(feature = "enrollment-test-transport")]
impl TestEnrollmentTransport {
    pub fn from_environment() -> Self {
        let create: std::collections::VecDeque<_> = std::env::var("GYROGNOME_TEST_CREATE")
            .unwrap_or_else(|_| "success".to_owned())
            .split(',')
            .map(|outcome| match outcome {
                "success" => CreateDelivery::Response("ok|73".to_owned()),
                "duplicate" => CreateDelivery::Response("That name is already taken.".to_owned()),
                _ => CreateDelivery::DeliveryFailed,
            })
            .collect();
        let report = match std::env::var("GYROGNOME_TEST_REPORT").as_deref() {
            Ok("rejected") => DeliveryOutcome::EndpointRejected,
            Ok("failed") => DeliveryOutcome::DeliveryFailed,
            _ => DeliveryOutcome::Delivered,
        };
        Self {
            create: std::cell::RefCell::new(create),
            report,
        }
    }
}

#[cfg(feature = "enrollment-test-transport")]
impl ReportTransport for TestEnrollmentTransport {
    fn deliver(&self, _: Url) -> DeliveryOutcome {
        if record_test_action("report") {
            self.report
        } else {
            DeliveryOutcome::DeliveryFailed
        }
    }
}

#[cfg(feature = "enrollment-test-transport")]
fn record_test_action(action: &str) -> bool {
    use std::io::Write;
    let Some(path) = std::env::var_os("GYROGNOME_TEST_ACTION_LOG") else {
        return true;
    };
    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .and_then(|mut file| writeln!(file, "{action}"))
        .is_ok()
}

#[cfg(feature = "enrollment-test-transport")]
impl GuildTransport for TestEnrollmentTransport {
    fn guild(&self, _: Url, _: &str, _: &str, _: &GuildResponseRules) -> GuildOutcome {
        if !record_test_action("guild") {
            return GuildOutcome::Indeterminate;
        }
        match std::env::var("GYROGNOME_TEST_GUILD_OUTCOME").as_deref() {
            Ok("accepted") => GuildOutcome::Accepted,
            Ok("rejected") => GuildOutcome::Rejected,
            _ => GuildOutcome::Indeterminate,
        }
    }
}

#[cfg(feature = "enrollment-test-transport")]
pub fn set_guild_for_test(
    store: &mut Store,
    id: &CharacterId,
    designation: &str,
    transport: &impl GuildTransport,
) -> Result<GuildResult, ReportingError> {
    let evidence = match std::env::var("GYROGNOME_TEST_GUILD_EVIDENCE").as_deref() {
        Ok("invalid") => GuildResponseRules::from_evidence("{}"),
        _ => GuildResponseRules::bundled(),
    };
    set_guild_with_evidence(store, id, designation, transport, evidence)
}

#[cfg(feature = "enrollment-test-transport")]
impl EnrollmentTransport for TestEnrollmentTransport {
    fn create(&self, _: Url) -> CreateDelivery {
        self.create
            .borrow_mut()
            .pop_front()
            .map(|outcome| match outcome {
                CreateDelivery::Response(response) => CreateDelivery::Response(response.clone()),
                CreateDelivery::DeliveryFailed => CreateDelivery::DeliveryFailed,
            })
            .unwrap_or(CreateDelivery::DeliveryFailed)
    }
}

pub fn classify_create_response(response: &str) -> CreateOutcome {
    let response = response.trim();
    let Some(passkey) = response
        .strip_prefix("ok|")
        .and_then(|value| value.parse::<i32>().ok())
        .filter(|passkey| *passkey != 0)
    else {
        return if duplicate_name_rejection(response) {
            CreateOutcome::DuplicateName
        } else {
            CreateOutcome::Incomplete
        };
    };
    CreateOutcome::Enrolled(passkey)
}

pub fn create(
    name: &str,
    realm: &str,
    transport: &impl EnrollmentTransport,
) -> Result<CreateOutcome, ReportingError> {
    let request = protocol::create_request(OFFICIAL_LEADERBOARD_HOST, name, realm);
    let request = official_endpoint(&request)?;
    Ok(match transport.create(request) {
        CreateDelivery::Response(response) => classify_create_response(&response),
        CreateDelivery::DeliveryFailed => CreateOutcome::Incomplete,
    })
}

#[derive(Debug)]
pub enum EnrollmentOutcome {
    Registered(Box<crate::runtime::ManagedCharacter>),
    DuplicateName,
}

pub fn enroll(
    store: &mut Store,
    draft: Character,
    transport: &impl EnrollmentTransport,
) -> Result<EnrollmentOutcome, ReportingError> {
    enroll_with_evidence(
        store,
        draft,
        transport,
        validate_bundled_enrollment_evidence(),
    )
}

#[cfg(feature = "enrollment-test-transport")]
pub fn enroll_for_test(
    store: &mut Store,
    draft: Character,
    transport: &impl EnrollmentTransport,
) -> Result<EnrollmentOutcome, ReportingError> {
    let evidence = match std::env::var("GYROGNOME_TEST_EVIDENCE").as_deref() {
        Ok("unavailable") => Err(EnrollmentEvidenceError::Unavailable),
        Ok("malformed") => Err(EnrollmentEvidenceError::Malformed),
        Ok("sensitive") => Err(EnrollmentEvidenceError::Sensitive),
        Ok("non-passing") => Err(EnrollmentEvidenceError::NotLiveOrPassing),
        _ => validate_bundled_enrollment_evidence(),
    };
    enroll_with_evidence(store, draft, transport, evidence)
}

fn enroll_with_evidence(
    store: &mut Store,
    draft: Character,
    transport: &impl EnrollmentTransport,
    evidence: Result<(), EnrollmentEvidenceError>,
) -> Result<EnrollmentOutcome, ReportingError> {
    evidence?;
    let passkey = match create(&draft.traits.name, OFFICIAL_REALM, transport)? {
        CreateOutcome::Enrolled(passkey) => passkey,
        CreateOutcome::DuplicateName => return Ok(EnrollmentOutcome::DuplicateName),
        CreateOutcome::Incomplete => return Err(ReportingError::IncompleteEnrollment),
    };
    let character = enrolled_character(draft, OFFICIAL_REALM, passkey)?;
    let request = protocol::progress_report(
        OFFICIAL_LEADERBOARD_HOST,
        &character,
        's',
        ReportFields {
            xp_position: character.progress.experience.position as u64,
            best_equipment: &character.bestequip,
            best_spell: &character.bestspell,
            best_stat: &character.beststat,
            best_plot: &character.plot.bestplot,
            motto: "",
        },
        passkey,
    )
    .map_err(|_| ReportingError::Construction)?;
    let request = official_endpoint(&request)?;
    match transport.deliver(request) {
        DeliveryOutcome::Delivered => Ok(EnrollmentOutcome::Registered(Box::new(
            store.register(&character)?,
        ))),
        DeliveryOutcome::EndpointRejected | DeliveryOutcome::DeliveryFailed => {
            Err(ReportingError::IncompleteEnrollment)
        }
    }
}

fn enrolled_character(
    mut character: Character,
    realm: &str,
    passkey: i32,
) -> Result<Character, ReportingError> {
    crate::simulation::update_beststat(&mut character);
    character.online = Some(OnlineMetadata {
        realm: realm.to_owned(),
        host: OFFICIAL_LEADERBOARD_HOST.to_owned(),
    });
    character.save_name = format!("{} [{realm}]", character.traits.name);
    let mut document =
        serde_json::to_value(&character).map_err(|_| ReportingError::Construction)?;
    document["online"]["passkey"] = serde_json::Value::from(passkey);
    character.document = document;
    Ok(character)
}

fn duplicate_name_rejection(response: &str) -> bool {
    let response = response.to_ascii_lowercase();
    [
        "name already",
        "name is already",
        "name has already",
        "duplicate name",
        "name is taken",
        "name taken",
    ]
    .iter()
    .any(|phrase| response.contains(phrase))
}

fn official_endpoint(request: &str) -> Result<Url, ReportingError> {
    let endpoint = Url::parse(request).map_err(|_| ReportingError::Construction)?;
    if endpoint.scheme() != "https"
        || endpoint.host_str() != Some("progressquest.com")
        || endpoint.path() != "/alpaquil.php"
        || endpoint.port().is_some()
        || !endpoint.username().is_empty()
        || endpoint.password().is_some()
        || endpoint.fragment().is_some()
    {
        return Err(ReportingError::UnofficialEndpoint);
    }
    Ok(endpoint)
}

fn browser_report_host(host: &str) -> Result<&'static str, ReportingError> {
    let mut endpoint = Url::parse(host).map_err(|_| ReportingError::Construction)?;
    if endpoint.scheme() == "http" {
        endpoint
            .set_scheme("https")
            .map_err(|_| ReportingError::Construction)?;
    }
    let endpoint = official_endpoint(endpoint.as_str())?;
    if endpoint.as_str().trim_end_matches('?') != OFFICIAL_LEADERBOARD_ENDPOINT {
        return Err(ReportingError::UnofficialEndpoint);
    }
    Ok(OFFICIAL_LEADERBOARD_HOST)
}

pub struct ReportResult {
    pub identity: CharacterIdentity,
    pub realm: String,
    pub outcome: DeliveryOutcome,
}

#[derive(Debug, Error)]
pub enum PublicLeaderboardError {
    #[error("no public leaderboard page is configured for this realm")]
    UnsupportedRealm,
    #[error("could not construct the public leaderboard page URL")]
    InvalidPageUrl(#[source] url::ParseError),
    #[error("could not open the public leaderboard page: {0}")]
    BrowserLaunch(#[source] std::io::Error),
}

pub fn public_leaderboard_url(
    realm: &str,
    display_name: &str,
) -> Result<Url, PublicLeaderboardError> {
    let page = match realm {
        "Alpaquil" => "https://progressquest.com/alpaquil.php",
        "Spoltog" => "https://progressquest.com/spoltog.php",
        "Pemptus" => "https://progressquest.com/pemptus.php",
        _ => return Err(PublicLeaderboardError::UnsupportedRealm),
    };
    let mut url = Url::parse(page).map_err(PublicLeaderboardError::InvalidPageUrl)?;
    url.query_pairs_mut().append_pair("name", display_name);
    Ok(url)
}

pub fn open_public_leaderboard(
    realm: &str,
    display_name: &str,
) -> Result<(), PublicLeaderboardError> {
    open_public_leaderboard_with(realm, display_name, |url| {
        let status = Command::new("xdg-open").arg(url.as_str()).status()?;
        if status.success() {
            Ok(())
        } else {
            Err(std::io::Error::other(
                "xdg-open exited with an unsuccessful status",
            ))
        }
    })
}

fn open_public_leaderboard_with(
    realm: &str,
    display_name: &str,
    open: impl FnOnce(&Url) -> std::io::Result<()>,
) -> Result<(), PublicLeaderboardError> {
    let url = public_leaderboard_url(realm, display_name)?;
    open(&url).map_err(PublicLeaderboardError::BrowserLaunch)
}

pub fn submit(
    store: &Store,
    id: &CharacterId,
    transport: &impl ReportTransport,
) -> Result<ReportResult, ReportingError> {
    if store.compatibility_profile(id)? == CompatibilityProfile::Desktop644 {
        return submit_desktop_manual(store, id, transport);
    }
    validate_bundled_enrollment_evidence()?;
    let target = store.online_action_target(id, DesktopOnlineOperation::ManualBrag, None)?;
    let mut state = target.state;
    crate::simulation::update_bestspell(&mut state);
    crate::simulation::update_beststat(&mut state);
    let host = state
        .online
        .as_ref()
        .map(|online| online.host.as_str())
        .ok_or(StorageError::ReportingIneligible)?;
    let host = browser_report_host(host)?;
    let realm = state
        .online
        .as_ref()
        .map(|online| online.realm.clone())
        .ok_or(StorageError::ReportingIneligible)?;
    let request = protocol::progress_report(
        host,
        &state,
        'b',
        ReportFields {
            xp_position: state.progress.experience.position as u64,
            best_equipment: &state.bestequip,
            best_spell: &state.bestspell,
            best_stat: &state.beststat,
            best_plot: &state.plot.bestplot,
            motto: &state.profile.motto,
        },
        target.passkey,
    )
    .map_err(|_| ReportingError::Construction)?;
    let request = official_endpoint(&request)?;
    Ok(ReportResult {
        identity: target.identity,
        realm,
        outcome: transport.deliver(request),
    })
}

pub fn submit_manual_brag(
    store: &Store,
    id: &CharacterId,
    transport: &impl ReportTransport,
) -> Result<ReportResult, ReportingError> {
    submit_manual_brag_with_opener(store, id, transport, open_public_leaderboard)
}

fn submit_manual_brag_with_opener(
    store: &Store,
    id: &CharacterId,
    transport: &impl ReportTransport,
    open: impl FnOnce(&str, &str) -> Result<(), PublicLeaderboardError>,
) -> Result<ReportResult, ReportingError> {
    let result = submit(store, id, transport)?;
    if let Err(error) = open(&result.realm, &result.identity.name) {
        eprintln!("Could not open the public leaderboard page: {error}");
    }
    Ok(result)
}

pub fn set_motto(
    store: &mut Store,
    id: &CharacterId,
    motto: &str,
    transport: &impl ReportTransport,
) -> Result<ReportResult, ReportingError> {
    if store.compatibility_profile(id)? == CompatibilityProfile::Desktop644 {
        return set_desktop_motto(store, id, motto, transport);
    }
    set_motto_with_evidence(
        store,
        id,
        motto,
        transport,
        validate_bundled_enrollment_evidence(),
    )
}

pub struct GuildResult {
    pub identity: CharacterIdentity,
    pub outcome: GuildOutcome,
}

pub fn set_guild(
    store: &mut Store,
    id: &CharacterId,
    designation: &str,
    transport: &impl GuildTransport,
) -> Result<GuildResult, ReportingError> {
    if store.compatibility_profile(id)? == CompatibilityProfile::Desktop644 {
        return set_desktop_guild(store, id, designation, transport);
    }
    set_guild_with_evidence(
        store,
        id,
        designation,
        transport,
        GuildResponseRules::bundled(),
    )
}

fn set_guild_with_evidence(
    store: &mut Store,
    id: &CharacterId,
    designation: &str,
    transport: &impl GuildTransport,
    evidence: Result<GuildResponseRules, crate::fixtures::GuildEvidenceError>,
) -> Result<GuildResult, ReportingError> {
    if designation.chars().any(char::is_control) {
        return Err(ReportingError::InvalidProfileText);
    }
    let rules = evidence?;
    let target =
        store.online_action_target(id, DesktopOnlineOperation::Guild, Some(designation))?;
    let state = &target.state;
    let host = state
        .online
        .as_ref()
        .map(|online| online.host.as_str())
        .ok_or(StorageError::ReportingIneligible)?;
    let host = browser_report_host(host)?;
    let request = protocol::guild_request(host, state, designation, target.passkey)
        .map_err(|_| ReportingError::Construction)?;
    let outcome = transport.guild(
        official_endpoint(&request)?,
        &state.profile.guild,
        designation,
        &rules,
    );
    if outcome == GuildOutcome::Accepted {
        let mut profile = state.profile.clone();
        profile.guild = designation.to_owned();
        store.replace_profile(id, &profile)?;
    }
    Ok(GuildResult {
        identity: target.identity,
        outcome,
    })
}

fn set_motto_with_evidence(
    store: &mut Store,
    id: &CharacterId,
    motto: &str,
    transport: &impl ReportTransport,
    evidence: Result<(), EnrollmentEvidenceError>,
) -> Result<ReportResult, ReportingError> {
    if motto.chars().any(char::is_control) {
        return Err(ReportingError::InvalidProfileText);
    }
    evidence?;
    let target = store.online_action_target(id, DesktopOnlineOperation::Motto, Some(motto))?;
    let mut state = target.state;
    crate::simulation::update_bestspell(&mut state);
    crate::simulation::update_beststat(&mut state);
    let host = state
        .online
        .as_ref()
        .map(|online| online.host.as_str())
        .ok_or(StorageError::ReportingIneligible)?;
    let host = browser_report_host(host)?;

    let mut profile = state.profile.clone();
    profile.motto = motto.to_owned();
    store.replace_profile(id, &profile)?;
    state.profile = profile;

    let request = protocol::progress_report(
        host,
        &state,
        'm',
        ReportFields {
            xp_position: state.progress.experience.position as u64,
            best_equipment: &state.bestequip,
            best_spell: &state.bestspell,
            best_stat: &state.beststat,
            best_plot: &state.plot.bestplot,
            motto,
        },
        target.passkey,
    )
    .map_err(|_| ReportingError::Construction)?;
    let request = official_endpoint(&request)?;
    Ok(ReportResult {
        identity: target.identity,
        realm: state
            .online
            .as_ref()
            .map(|online| online.realm.clone())
            .ok_or(StorageError::ReportingIneligible)?,
        outcome: transport.deliver(request),
    })
}

/// Delivers one persisted worker trace event through the official endpoint.
///
/// The caller must own the managed-character worker lock and persist the
/// canonical successor state before calling this function.
pub(crate) fn submit_event(
    store: &Store,
    id: &CharacterId,
    event: &ReportEvent,
    transport: &(impl ReportTransport + ?Sized),
) -> Result<ReportResult, ReportingError> {
    if !matches!(
        event.trigger,
        ReportTrigger::LevelUp | ReportTrigger::ActCompletion
    ) {
        return Err(ReportingError::Construction);
    }

    validate_bundled_enrollment_evidence()?;
    let target = store.reporting_target_for_worker(id)?;
    let state = &event.snapshot.character;
    let host = state
        .online
        .as_ref()
        .map(|online| online.host.as_str())
        .ok_or(StorageError::ReportingIneligible)?;
    let host = browser_report_host(host)?;
    let request = protocol::progress_report(
        host,
        state,
        event.trigger.code(),
        ReportFields {
            xp_position: state.progress.experience.position as u64,
            best_equipment: &state.bestequip,
            best_spell: &state.bestspell,
            best_stat: &state.beststat,
            best_plot: &state.plot.bestplot,
            motto: &target.state.profile.motto,
        },
        target.passkey,
    )
    .map_err(|_| ReportingError::Construction)?;
    let request = official_endpoint(&request)?;
    Ok(ReportResult {
        identity: target.identity,
        realm: state
            .online
            .as_ref()
            .map(|online| online.realm.clone())
            .ok_or(StorageError::ReportingIneligible)?,
        outcome: transport.deliver(request),
    })
}

/// Enters the serialized desktop worker-report boundary after callback state
/// has been committed. Desktop delivery remains closed until its independent
/// evidence, eligibility, protocol, and transport contracts are implemented.
pub(crate) fn submit_desktop_event(
    store: &Store,
    id: &CharacterId,
    event: &DesktopReportSnapshot,
    transport: &(impl ReportTransport + ?Sized),
) -> Result<ReportResult, ReportingError> {
    let operation = match event.trigger {
        crate::desktop_simulation::DesktopReportTrigger::Level => {
            DesktopOnlineOperation::AutomaticLevel
        }
        crate::desktop_simulation::DesktopReportTrigger::Act => {
            DesktopOnlineOperation::AutomaticAct
        }
    };
    let target = store.desktop_reporting_target_for_worker(id, operation, &event.state)?;
    let authentication = DesktopAccountAuthentication::new(
        &target.authentication.account,
        &target.authentication.password,
    );
    let request = desktop_report_for_snapshot(
        event,
        &target.authentication.realm,
        &target.profile.motto,
        target.authentication.passkey,
        &authentication,
    )
    .map_err(|_| ReportingError::DesktopConstruction)?;
    let response =
        deliver_desktop_report(transport, &target.authentication, request.encoded_query());
    Ok(ReportResult {
        identity: target.identity,
        realm: target.authentication.realm.clone(),
        outcome: response,
    })
}

fn submit_desktop_manual(
    store: &Store,
    id: &CharacterId,
    transport: &impl ReportTransport,
) -> Result<ReportResult, ReportingError> {
    let target =
        store.desktop_online_action_target(id, DesktopOnlineOperation::ManualBrag, None)?;
    let authentication = DesktopAccountAuthentication::new(
        &target.authentication.account,
        &target.authentication.password,
    );
    let request = desktop_report(
        &target.state,
        DesktopReportOperation::Manual,
        &target.authentication.realm,
        &target.profile.motto,
        target.authentication.passkey,
        &authentication,
    )
    .map_err(|_| ReportingError::DesktopConstruction)?;
    Ok(ReportResult {
        identity: target.identity,
        realm: target.authentication.realm.clone(),
        outcome: deliver_desktop_report(transport, &target.authentication, request.encoded_query()),
    })
}

fn set_desktop_motto(
    store: &mut Store,
    id: &CharacterId,
    motto: &str,
    transport: &impl ReportTransport,
) -> Result<ReportResult, ReportingError> {
    let target =
        store.desktop_online_action_target(id, DesktopOnlineOperation::Motto, Some(motto))?;
    let mut profile = target.profile.clone();
    profile.motto = motto.to_owned();
    store.replace_profile(id, &profile)?;
    let mut state = target.state;
    state.profile.motto = motto.to_owned();
    let authentication = DesktopAccountAuthentication::new(
        &target.authentication.account,
        &target.authentication.password,
    );
    let request = desktop_report(
        &state,
        DesktopReportOperation::Motto,
        &target.authentication.realm,
        motto,
        target.authentication.passkey,
        &authentication,
    )
    .map_err(|_| ReportingError::DesktopConstruction)?;
    Ok(ReportResult {
        identity: target.identity,
        realm: target.authentication.realm.clone(),
        outcome: deliver_desktop_report(transport, &target.authentication, request.encoded_query()),
    })
}

fn set_desktop_guild(
    store: &mut Store,
    id: &CharacterId,
    designation: &str,
    transport: &impl GuildTransport,
) -> Result<GuildResult, ReportingError> {
    let target =
        store.desktop_online_action_target(id, DesktopOnlineOperation::Guild, Some(designation))?;
    let authentication = DesktopAccountAuthentication::new(
        &target.authentication.account,
        &target.authentication.password,
    );
    let request = desktop_guild_request(
        &target.state,
        &target.authentication.realm,
        designation,
        target.authentication.passkey,
        &authentication,
    )
    .map_err(|_| ReportingError::DesktopConstruction)?;
    let endpoint = resolve_verified_desktop_endpoint(
        &target.authentication.realm,
        &target.authentication.endpoint,
    )?;
    let credentials = DesktopTransportCredentials::new(
        &target.authentication.account,
        &target.authentication.password,
    );
    let authorization = credentials.authorization_header(endpoint.credential_mode())?;
    let passkey = target.authentication.passkey.to_string();
    let operation = if designation.is_empty() {
        DesktopGuildOperation::Leave
    } else {
        DesktopGuildOperation::JoinOrChange
    };
    let rules = DesktopGuildResponseRules::production_for_import(
        &target.authentication.realm,
        operation,
        &target.adaptations,
    )?;
    let desktop_outcome = apply_desktop_guild_action(
        &mut target.state.profile.clone(),
        Some(designation),
        |prior, submitted| {
            let dynamic_values = DesktopGuildFingerprintValues {
                character_name: &target.identity.name,
                account: &target.authentication.account,
                password: &target.authentication.password,
                authorization: authorization.as_deref().unwrap_or(""),
                passkey: &passkey,
                prior_guild: prior,
                submitted_guild: submitted,
            };
            transport
                .guild_desktop(&endpoint, &request.encoded_query(), &credentials)
                .map(|response| rules.classify(response.status, &response.body, &dynamic_values))
                .unwrap_or(DesktopGuildOutcome::Indeterminate)
        },
    )?;
    let (desktop_outcome, reconciled_guild) =
        reconcile_desktop_guild_outcome(desktop_outcome, designation, || {
            transport.desktop_public_guild(&endpoint, &target.identity.name)
        });
    let outcome = match desktop_outcome {
        DesktopGuildOutcome::Accepted => GuildOutcome::Accepted,
        DesktopGuildOutcome::Rejected => GuildOutcome::Rejected,
        DesktopGuildOutcome::Indeterminate | DesktopGuildOutcome::Cancelled => {
            GuildOutcome::Indeterminate
        }
    };
    if outcome == GuildOutcome::Accepted {
        let mut profile = target.profile;
        profile.guild = reconciled_guild.unwrap_or_else(|| designation.to_owned());
        store.replace_profile(id, &profile)?;
    }
    Ok(GuildResult {
        identity: target.identity,
        outcome,
    })
}

fn reconcile_desktop_guild_outcome(
    outcome: DesktopGuildOutcome,
    submitted: &str,
    observe_public_guild: impl FnOnce() -> Result<Option<String>, DesktopTransportError>,
) -> (DesktopGuildOutcome, Option<String>) {
    if outcome != DesktopGuildOutcome::Indeterminate {
        return (outcome, None);
    }
    let Ok(observed) = observe_public_guild() else {
        return (outcome, None);
    };
    let matches = public_guild_matches(observed.as_deref(), submitted);
    matches
        .then(|| {
            (
                DesktopGuildOutcome::Accepted,
                Some(observed.unwrap_or_default()),
            )
        })
        .unwrap_or((outcome, None))
}

fn deliver_desktop_report(
    transport: &(impl ReportTransport + ?Sized),
    authentication: &crate::runtime::DesktopAuthentication,
    encoded_query: String,
) -> DeliveryOutcome {
    let target =
        match resolve_verified_desktop_endpoint(&authentication.realm, &authentication.endpoint) {
            Ok(target) => target,
            Err(_) => return DeliveryOutcome::DeliveryFailed,
        };
    let credentials =
        DesktopTransportCredentials::new(&authentication.account, &authentication.password);
    match transport.deliver_desktop(&target, &encoded_query, &credentials) {
        Ok(response) if (200..300).contains(&response.status) => DeliveryOutcome::Delivered,
        Ok(_) => DeliveryOutcome::EndpointRejected,
        Err(_) => DeliveryOutcome::DeliveryFailed,
    }
}

#[cfg(test)]
mod tests {
    use std::{
        cell::{Cell, RefCell},
        fs,
        path::{Path, PathBuf},
    };

    use base64::{Engine, engine::general_purpose::STANDARD};
    use uuid::Uuid;

    use super::*;
    use crate::{
        checkpoint, newguy,
        runtime::{DesktopAuthentication, Store, Worker},
        save,
        simulation::{ReportTrigger, advance_with_trace},
    };

    #[test]
    fn public_leaderboard_urls_use_fixed_realm_pages_and_encode_names() {
        for (realm, page) in [
            ("Alpaquil", "alpaquil.php"),
            ("Spoltog", "spoltog.php"),
            ("Pemptus", "pemptus.php"),
        ] {
            let url = public_leaderboard_url(realm, "A name & ?#").unwrap();
            assert_eq!(
                url.as_str(),
                format!("https://progressquest.com/{page}?name=A+name+%26+%3F%23")
            );
            assert_eq!(
                url.query_pairs().collect::<Vec<_>>(),
                [("name".into(), "A name & ?#".into())]
            );
        }
        assert!(matches!(
            public_leaderboard_url("Other", "Name"),
            Err(PublicLeaderboardError::UnsupportedRealm)
        ));
    }

    #[test]
    fn public_leaderboard_opening_reports_launcher_failures_separately() {
        let error = open_public_leaderboard_with("Spoltog", "A name", |_| {
            Err(std::io::Error::other("launcher failed"))
        })
        .unwrap_err();
        assert!(matches!(error, PublicLeaderboardError::BrowserLaunch(_)));
    }

    struct TestDirectory(PathBuf);
    impl TestDirectory {
        fn new() -> Self {
            let path = Path::new("target/reporting-tests").join(Uuid::new_v4().to_string());
            fs::create_dir_all(&path).unwrap();
            Self(path)
        }
    }
    impl Drop for TestDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    struct FakeTransport {
        create: CreateDelivery,
        report: DeliveryOutcome,
    }

    struct DesktopRecordingTransport {
        queries: RefCell<Vec<String>>,
        response: RefCell<Option<DesktopHttpResponse>>,
    }

    impl ReportTransport for DesktopRecordingTransport {
        fn deliver(&self, _request: Url) -> DeliveryOutcome {
            panic!("browser delivery was used for a desktop request")
        }

        fn deliver_desktop(
            &self,
            target: &VerifiedDesktopEndpoint,
            encoded_query: &str,
            credentials: &DesktopTransportCredentials,
        ) -> Result<DesktopHttpResponse, DesktopTransportError> {
            let debug = format!("{target:?} {credentials:?}");
            assert!(!debug.contains("desktop-account"));
            assert!(!debug.contains("desktop-password"));
            self.queries.borrow_mut().push(encoded_query.to_owned());
            self.response
                .borrow_mut()
                .take()
                .ok_or(DesktopTransportError::DeliveryFailed)
        }
    }

    impl ReportTransport for FakeTransport {
        fn deliver(&self, request: Url) -> DeliveryOutcome {
            assert_eq!(request.scheme(), "https");
            assert_eq!(request.host_str(), Some("progressquest.com"));
            assert_eq!(request.path(), "/alpaquil.php");
            assert!(matches!(
                request
                    .query_pairs()
                    .find(|(key, _)| key == "t")
                    .map(|(_, value)| value.into_owned()),
                Some(value) if value == "b" || value == "s"
            ));
            self.report
        }
    }

    impl EnrollmentTransport for FakeTransport {
        fn create(&self, request: Url) -> CreateDelivery {
            assert_eq!(request.scheme(), "https");
            assert_eq!(request.host_str(), Some("progressquest.com"));
            assert_eq!(request.path(), "/alpaquil.php");
            match &self.create {
                CreateDelivery::Response(response) => CreateDelivery::Response(response.clone()),
                CreateDelivery::DeliveryFailed => CreateDelivery::DeliveryFailed,
            }
        }
    }

    struct Numbers(u32);

    impl crate::newguy::RandomSource for Numbers {
        fn next_u32(&mut self) -> Result<u32, crate::newguy::NewGuyError> {
            let value = self.0;
            self.0 = self.0.wrapping_add(1);
            Ok(value)
        }
    }

    #[test]
    fn desktop_delivery_uses_the_verified_spoltog_transport_contract() {
        let transport = DesktopRecordingTransport {
            queries: RefCell::new(Vec::new()),
            response: RefCell::new(Some(DesktopHttpResponse {
                status: 200,
                redirect: None,
                body: Vec::new(),
            })),
        };
        let authentication = DesktopAuthentication {
            passkey: 42,
            realm: "Spoltog".to_owned(),
            endpoint: "http://progressquest.com/spoltog.php?".to_owned(),
            account: "desktop-account".to_owned(),
            password: "desktop-password".to_owned(),
        };

        assert_eq!(
            deliver_desktop_report(&transport, &authentication, "cmd=b&rev=8&p=1".to_owned()),
            DeliveryOutcome::Delivered
        );
        assert_eq!(transport.queries.borrow().as_slice(), ["cmd=b&rev=8&p=1"]);
    }

    struct RecordingTransport {
        requests: RefCell<Vec<Vec<String>>>,
    }

    impl ReportTransport for RecordingTransport {
        fn deliver(&self, request: Url) -> DeliveryOutcome {
            assert_progress_request(&request, 73);
            assert_eq!(
                request.query_pairs().find(|(key, _)| key == "k").unwrap().1,
                "WIS 80"
            );
            self.requests.borrow_mut().push(
                request
                    .query_pairs()
                    .map(|(key, _)| key.into_owned())
                    .collect(),
            );
            DeliveryOutcome::Delivered
        }
    }

    impl EnrollmentTransport for RecordingTransport {
        fn create(&self, request: Url) -> CreateDelivery {
            self.requests.borrow_mut().push(
                request
                    .query_pairs()
                    .map(|(key, _)| key.into_owned())
                    .collect(),
            );
            CreateDelivery::Response("ok|73".to_owned())
        }
    }

    struct CountingTransport {
        create: CreateDelivery,
        report: DeliveryOutcome,
        calls: RefCell<Vec<&'static str>>,
    }

    impl ReportTransport for CountingTransport {
        fn deliver(&self, _: Url) -> DeliveryOutcome {
            self.calls.borrow_mut().push("report");
            self.report
        }
    }

    impl EnrollmentTransport for CountingTransport {
        fn create(&self, _: Url) -> CreateDelivery {
            self.calls.borrow_mut().push("create");
            match &self.create {
                CreateDelivery::Response(response) => CreateDelivery::Response(response.clone()),
                CreateDelivery::DeliveryFailed => CreateDelivery::DeliveryFailed,
            }
        }
    }

    fn draft() -> Character {
        newguy::generate(
            &newguy::Selection {
                name: "Online Hero".to_owned(),
                race: "Gyrognome".to_owned(),
                class: "Robot Monk".to_owned(),
            },
            &crate::ruleset::BUNDLED,
            &mut Numbers(1),
        )
        .unwrap()
    }

    #[test]
    fn classifies_only_safe_enrollment_outcomes() {
        assert_eq!(
            classify_create_response("ok|73"),
            CreateOutcome::Enrolled(73)
        );
        assert_eq!(
            classify_create_response("That name is already taken."),
            CreateOutcome::DuplicateName
        );
        for response in ["ok|0", "ok|not-a-number", "unexpected server output"] {
            assert_eq!(
                classify_create_response(response),
                CreateOutcome::Incomplete
            );
        }
    }

    #[test]
    fn indeterminate_desktop_guild_outcomes_use_public_verification_once() {
        let mutation_calls = Cell::new(0);
        let public_calls = Cell::new(0);
        let mut profile = crate::desktop_save::DesktopValidatedProfile {
            motto: String::new(),
            guild: "Old Guild".to_owned(),
        };
        let response_outcome =
            apply_desktop_guild_action(&mut profile, Some("beerguild"), |prior, submitted| {
                mutation_calls.set(mutation_calls.get() + 1);
                assert_eq!(prior, "Old Guild");
                assert_eq!(submitted, "beerguild");
                DesktopGuildOutcome::Indeterminate
            })
            .unwrap();
        let (outcome, canonical_guild) =
            reconcile_desktop_guild_outcome(response_outcome, "beerguild", || {
                public_calls.set(public_calls.get() + 1);
                Ok(Some("BEERguild".to_owned()))
            });
        assert_eq!(outcome, DesktopGuildOutcome::Accepted);
        profile.guild = canonical_guild.unwrap();
        assert_eq!(profile.guild, "BEERguild");
        assert_eq!(mutation_calls.get(), 1);
        assert_eq!(public_calls.get(), 1);

        assert_eq!(
            reconcile_desktop_guild_outcome(DesktopGuildOutcome::Indeterminate, "", || Ok(None)),
            (DesktopGuildOutcome::Accepted, Some(String::new()))
        );
        assert_eq!(
            reconcile_desktop_guild_outcome(
                DesktopGuildOutcome::Indeterminate,
                "Other Guild",
                || Ok(Some("BEERguild".to_owned()))
            ),
            (DesktopGuildOutcome::Indeterminate, None)
        );
        assert_eq!(
            reconcile_desktop_guild_outcome(
                DesktopGuildOutcome::Indeterminate,
                "BEERguild",
                || { Ok(None) }
            ),
            (DesktopGuildOutcome::Indeterminate, None)
        );
        assert_eq!(
            reconcile_desktop_guild_outcome(
                DesktopGuildOutcome::Indeterminate,
                "BEERguild",
                || Err(DesktopTransportError::DeliveryFailed)
            ),
            (DesktopGuildOutcome::Indeterminate, None)
        );
        assert_eq!(
            reconcile_desktop_guild_outcome(DesktopGuildOutcome::Rejected, "BEERguild", || {
                panic!("definitive response attempted public verification")
            }),
            (DesktopGuildOutcome::Rejected, None)
        );
        assert_eq!(
            reconcile_desktop_guild_outcome(DesktopGuildOutcome::Accepted, "BEERguild", || {
                panic!("accepted response attempted public verification")
            }),
            (DesktopGuildOutcome::Accepted, None)
        );
    }

    #[test]
    fn create_uses_only_the_official_endpoint() {
        assert_eq!(
            create(
                "Safe Name",
                "Alpaquil",
                &FakeTransport {
                    create: CreateDelivery::Response("ok|73".to_owned()),
                    report: DeliveryOutcome::Delivered,
                }
            )
            .unwrap(),
            CreateOutcome::Enrolled(73)
        );
    }

    #[test]
    fn keeps_enrollment_credentials_only_in_the_private_source_document() {
        let character = newguy::generate(
            &newguy::Selection {
                name: "Online Hero".to_owned(),
                race: "Gyrognome".to_owned(),
                class: "Robot Monk".to_owned(),
            },
            &crate::ruleset::BUNDLED,
            &mut Numbers(1),
        )
        .unwrap();
        let character = enrolled_character(character, "Alpaquil", 73).unwrap();

        assert_eq!(character.online.as_ref().unwrap().realm, "Alpaquil");
        assert_eq!(
            character.online.as_ref().unwrap().host,
            OFFICIAL_LEADERBOARD_HOST
        );
        assert_eq!(character.document["online"]["passkey"], 73);
        let canonical = serde_json::to_value(&character).unwrap();
        assert_eq!(canonical["online"]["realm"], "Alpaquil");
        assert_eq!(canonical["online"]["host"], OFFICIAL_LEADERBOARD_HOST);
        assert!(canonical["online"].get("passkey").is_none());
    }

    #[test]
    fn enrolls_before_registering_with_a_browser_ordered_initial_report() {
        let directory = TestDirectory::new();
        let mut store = Store::open_at(&directory.0).unwrap();
        let mut draft = newguy::generate(
            &newguy::Selection {
                name: "Online Hero".to_owned(),
                race: "Gyrognome".to_owned(),
                class: "Robot Monk".to_owned(),
            },
            &crate::ruleset::BUNDLED,
            &mut Numbers(1),
        )
        .unwrap();
        draft.stats.best = "STR".to_owned();
        draft.beststat = "STR 1".to_owned();
        draft.stats.wisdom = 80.0;
        let transport = RecordingTransport {
            requests: RefCell::new(Vec::new()),
        };

        let EnrollmentOutcome::Registered(registered) =
            enroll(&mut store, draft, &transport).unwrap()
        else {
            panic!("successful enrollment must register");
        };

        assert_eq!(
            registered.state.online.as_ref().unwrap().realm,
            OFFICIAL_REALM
        );
        assert_eq!(registered.state.stats.best, "WIS");
        assert_eq!(registered.state.beststat, "WIS 80");
        let private = store.original_document(&registered.id).unwrap();
        assert_eq!(private["online"]["passkey"], 73);
        assert!(
            serde_json::to_string(&registered)
                .unwrap()
                .contains("Online Hero")
        );
        assert!(
            !serde_json::to_string(&registered)
                .unwrap()
                .contains("passkey")
        );
        assert_eq!(
            *transport.requests.borrow(),
            vec![
                vec!["cmd", "name", "realm", "rev"],
                vec![
                    "cmd", "t", "n", "r", "c", "l", "x", "i", "z", "k", "a", "h", "rev", "p", "m",
                ],
            ]
        );
    }

    #[test]
    fn evidence_failures_stop_enrollment_before_transport() {
        let draft = draft();
        for evidence in [
            EnrollmentEvidenceError::Unavailable,
            EnrollmentEvidenceError::Malformed,
            EnrollmentEvidenceError::Sensitive,
            EnrollmentEvidenceError::NotLiveOrPassing,
        ] {
            let directory = TestDirectory::new();
            let mut store = Store::open_at(&directory.0).unwrap();
            let transport = RecordingTransport {
                requests: RefCell::new(Vec::new()),
            };

            assert!(matches!(
                enroll_with_evidence(&mut store, draft.clone(), &transport, Err(evidence)),
                Err(ReportingError::Evidence(_))
            ));
            assert!(transport.requests.borrow().is_empty());
            assert!(store.list().unwrap().is_empty());
        }
    }

    #[test]
    fn duplicate_name_does_not_report_or_register() {
        let directory = TestDirectory::new();
        let mut store = Store::open_at(&directory.0).unwrap();
        let transport = CountingTransport {
            create: CreateDelivery::Response("That name is already taken.".to_owned()),
            report: DeliveryOutcome::Delivered,
            calls: RefCell::new(Vec::new()),
        };

        assert!(matches!(
            enroll(&mut store, draft(), &transport),
            Ok(EnrollmentOutcome::DuplicateName)
        ));
        assert_eq!(*transport.calls.borrow(), ["create"]);
        assert!(store.list().unwrap().is_empty());
    }

    #[test]
    fn incomplete_enrollment_never_retries_or_registers() {
        for (create, report, calls) in [
            (
                CreateDelivery::DeliveryFailed,
                DeliveryOutcome::Delivered,
                vec!["create"],
            ),
            (
                CreateDelivery::Response("private response body".to_owned()),
                DeliveryOutcome::Delivered,
                vec!["create"],
            ),
            (
                CreateDelivery::Response("ok|73".to_owned()),
                DeliveryOutcome::EndpointRejected,
                vec!["create", "report"],
            ),
            (
                CreateDelivery::Response("ok|73".to_owned()),
                DeliveryOutcome::DeliveryFailed,
                vec!["create", "report"],
            ),
        ] {
            let directory = TestDirectory::new();
            let mut store = Store::open_at(&directory.0).unwrap();
            let transport = CountingTransport {
                create,
                report,
                calls: RefCell::new(Vec::new()),
            };

            let error = enroll(&mut store, draft(), &transport).unwrap_err();
            assert_eq!(
                error.to_string(),
                "online enrollment is incomplete; no local character was registered"
            );
            assert_eq!(*transport.calls.borrow(), calls);
            assert!(store.list().unwrap().is_empty());
        }
    }

    fn assert_progress_request(request: &Url, passkey: i32) {
        assert_eq!(
            request.as_str().split('?').next(),
            Some(OFFICIAL_LEADERBOARD_ENDPOINT)
        );
        assert_eq!(
            request
                .query_pairs()
                .map(|(key, _)| key.into_owned())
                .collect::<Vec<_>>(),
            [
                "cmd", "t", "n", "r", "c", "l", "x", "i", "z", "k", "a", "h", "rev", "p", "m",
            ]
        );
        let (unsigned, _) = request.as_str().split_once("&p=").unwrap();
        let signature = request
            .query_pairs()
            .find(|(key, _)| key == "p")
            .unwrap()
            .1
            .parse::<i32>()
            .unwrap();
        assert_eq!(signature, protocol::validator(unsigned, passkey));
    }

    struct DerivedFieldsTransport {
        specialty: RefCell<Option<String>>,
        prime_stat: RefCell<Option<String>>,
    }

    impl ReportTransport for DerivedFieldsTransport {
        fn deliver(&self, request: Url) -> DeliveryOutcome {
            assert_progress_request(&request, 4242);
            *self.specialty.borrow_mut() = request
                .query_pairs()
                .find(|(key, _)| key == "z")
                .map(|(_, value)| value.into_owned());
            *self.prime_stat.borrow_mut() = request
                .query_pairs()
                .find(|(key, _)| key == "k")
                .map(|(_, value)| value.into_owned());
            DeliveryOutcome::Delivered
        }
    }

    fn registered_store() -> (TestDirectory, Store, CharacterId) {
        let directory = TestDirectory::new();
        let mut character = save::import_text(
            &STANDARD.encode(include_str!("../tests/fixtures/reference-save.json")),
        )
        .unwrap();
        character.online.as_mut().unwrap().host = format!("{OFFICIAL_LEADERBOARD_ENDPOINT}?");
        let mut store = Store::open_at(&directory.0).unwrap();
        let id = store.register(&character).unwrap().id;
        (directory, store, id)
    }

    #[test]
    fn browser_report_host_accepts_only_the_official_endpoint() {
        for host in [
            "http://progressquest.com/alpaquil.php",
            "http://progressquest.com/alpaquil.php?",
            OFFICIAL_LEADERBOARD_ENDPOINT,
            OFFICIAL_LEADERBOARD_HOST,
        ] {
            assert_eq!(
                browser_report_host(host).unwrap(),
                OFFICIAL_LEADERBOARD_HOST
            );
        }
        for host in [
            "http://example.invalid/alpaquil.php?",
            "https://progressquest.com.example.invalid/alpaquil.php?",
            "http://progressquest.com/spoltog.php?",
            "http://progressquest.com:8080/alpaquil.php?",
            "http://user@progressquest.com/alpaquil.php?",
            "http://user:password@progressquest.com/alpaquil.php?",
            "http://progressquest.com/alpaquil.php?cmd=create",
            "http://progressquest.com/alpaquil.php?#fragment",
            "ftp://progressquest.com/alpaquil.php?",
        ] {
            assert!(matches!(
                browser_report_host(host),
                Err(ReportingError::UnofficialEndpoint)
            ));
        }
        assert!(matches!(
            browser_report_host("not a URL"),
            Err(ReportingError::Construction)
        ));
        assert!(matches!(
            official_endpoint("http://progressquest.com/alpaquil.php?cmd=b"),
            Err(ReportingError::UnofficialEndpoint)
        ));
    }

    #[test]
    fn imported_browser_endpoints_use_https_for_all_online_actions() {
        for host in [
            "http://progressquest.com/alpaquil.php?",
            "http://progressquest.com/alpaquil.php",
            OFFICIAL_LEADERBOARD_ENDPOINT,
            OFFICIAL_LEADERBOARD_HOST,
        ] {
            let directory = TestDirectory::new();
            let mut document: serde_json::Value =
                serde_json::from_str(include_str!("../tests/fixtures/reference-save.json"))
                    .unwrap();
            document["online"]["host"] = host.into();
            let character =
                save::import_text(&STANDARD.encode(serde_json::to_vec(&document).unwrap()))
                    .unwrap();
            let mut store = Store::open_at(&directory.0).unwrap();
            let id = store.register(&character).unwrap().id;
            let transport = ProfileTransport {
                requests: RefCell::new(Vec::new()),
                outcome: DeliveryOutcome::Delivered,
            };
            let guild_transport = GuildRecorder {
                calls: RefCell::new(Vec::new()),
                outcome: GuildOutcome::Accepted,
            };

            assert_eq!(
                submit(&store, &id, &transport).unwrap().outcome,
                DeliveryOutcome::Delivered
            );
            assert_eq!(
                set_motto(&mut store, &id, "Legacy motto", &transport)
                    .unwrap()
                    .outcome,
                DeliveryOutcome::Delivered
            );
            assert_eq!(
                set_guild(&mut store, &id, "Legacy guild", &guild_transport)
                    .unwrap()
                    .outcome,
                GuildOutcome::Accepted
            );

            let mut initial =
                checkpoint::load(Path::new("tests/fixtures/checkpoint-level-up.json"))
                    .unwrap()
                    .initial;
            initial.online = character.online.clone();
            initial.queue = vec!["plot|1|Loading".to_owned()];
            let trace = advance_with_trace(&initial, &crate::ruleset::BUNDLED, 1_000, "").unwrap();
            store.replace_state(&id, &trace.state).unwrap();
            assert_eq!(
                trace
                    .events
                    .iter()
                    .map(|event| event.trigger)
                    .collect::<Vec<_>>(),
                [ReportTrigger::LevelUp, ReportTrigger::ActCompletion]
            );
            for event in &trace.events {
                assert_eq!(
                    submit_event(&store, &id, event, &transport)
                        .unwrap()
                        .outcome,
                    DeliveryOutcome::Delivered
                );
            }
            assert_eq!(
                transport
                    .requests
                    .borrow()
                    .iter()
                    .map(|(trigger, _)| trigger.as_str())
                    .collect::<Vec<_>>(),
                ["b", "m", "l", "a"]
            );
            assert_eq!(guild_transport.calls.borrow().len(), 1);
            assert_eq!(store.get(&id).unwrap().state.online.unwrap().host, host);
        }
    }

    #[test]
    fn delivers_one_manual_brag_with_safe_outcomes() {
        for expected in [
            DeliveryOutcome::Delivered,
            DeliveryOutcome::EndpointRejected,
            DeliveryOutcome::DeliveryFailed,
        ] {
            let (_directory, store, id) = registered_store();
            let opened = Cell::new(false);
            let result = submit_manual_brag_with_opener(
                &store,
                &id,
                &FakeTransport {
                    create: CreateDelivery::DeliveryFailed,
                    report: expected,
                },
                |realm, name| {
                    assert_eq!(realm, "Alpaquil");
                    assert_eq!(name, "Reference Hero");
                    opened.set(true);
                    Err(PublicLeaderboardError::BrowserLaunch(
                        std::io::Error::other("test launcher failure"),
                    ))
                },
            )
            .unwrap();
            assert_eq!(result.outcome, expected);
            assert_eq!(result.identity.name, "Reference Hero");
            assert!(opened.get());
        }
    }

    #[test]
    fn manual_brag_uses_persisted_state_without_interrupting_active_worker() {
        let (directory, mut store, id) = registered_store();
        store
            .replace_profile(
                &id,
                &crate::state::OnlineProfile {
                    motto: "Running motto".to_owned(),
                    guild: "Running guild".to_owned(),
                },
            )
            .unwrap();
        let before = serde_json::to_value(store.get(&id).unwrap().state).unwrap();
        let worker = Worker::start(Store::open_at(&directory.0).unwrap(), id.clone()).unwrap();
        let transport = MottoTransport {
            mottos: RefCell::new(Vec::new()),
        };

        let result = submit(&store, &id, &transport).unwrap();

        assert_eq!(result.outcome, DeliveryOutcome::Delivered);
        assert_eq!(*transport.mottos.borrow(), ["Running motto".to_owned()]);
        assert!(store.is_owned(&id).unwrap());
        assert_eq!(
            serde_json::to_value(store.get(&id).unwrap().state).unwrap(),
            before
        );
        drop(worker);
    }

    #[test]
    fn manual_brag_derives_specialty_and_prime_stat_before_constructing_the_report() {
        let (_directory, mut store, id) = registered_store();
        let mut state = store.get(&id).unwrap().state;
        state.bestspell = "Stale Specialty".to_owned();
        state.stats.best = "STR".to_owned();
        state.beststat = "STR 1".to_owned();
        state.stats.wisdom = 80.0;
        store.replace_state(&id, &state).unwrap();
        let transport = DerivedFieldsTransport {
            specialty: RefCell::new(None),
            prime_stat: RefCell::new(None),
        };

        submit(&store, &id, &transport).unwrap();

        assert_eq!(
            *transport.specialty.borrow(),
            Some("Hastiness II".to_owned())
        );
        assert_eq!(*transport.prime_stat.borrow(), Some("WIS 80".to_owned()));
        assert_eq!(
            serde_json::to_value(store.get(&id).unwrap().state).unwrap(),
            serde_json::to_value(&state).unwrap(),
            "report preparation must not overwrite a worker's canonical state"
        );
    }

    #[test]
    fn motto_change_derives_specialty_and_prime_stat_before_constructing_the_report() {
        let (_directory, mut store, id) = registered_store();
        let mut state = store.get(&id).unwrap().state;
        state.bestspell = "Stale Specialty".to_owned();
        state.stats.best = "STR".to_owned();
        state.beststat = "STR 1".to_owned();
        state.stats.wisdom = 80.0;
        store.replace_state(&id, &state).unwrap();
        let transport = DerivedFieldsTransport {
            specialty: RefCell::new(None),
            prime_stat: RefCell::new(None),
        };

        set_motto(&mut store, &id, "Current prime stat", &transport).unwrap();

        assert_eq!(
            *transport.specialty.borrow(),
            Some("Hastiness II".to_owned())
        );
        assert_eq!(*transport.prime_stat.borrow(), Some("WIS 80".to_owned()));
        state.profile.motto = "Current prime stat".to_owned();
        assert_eq!(
            serde_json::to_value(store.get(&id).unwrap().state).unwrap(),
            serde_json::to_value(&state).unwrap(),
            "motto changes must persist only the profile, not overwrite worker state"
        );
    }

    #[test]
    fn manual_brag_uses_the_current_persisted_motto() {
        let (_directory, mut store, id) = registered_store();
        store
            .replace_profile(
                &id,
                &crate::state::OnlineProfile {
                    motto: "Persisted manual motto".to_owned(),
                    guild: String::new(),
                },
            )
            .unwrap();
        let transport = MottoTransport {
            mottos: RefCell::new(Vec::new()),
        };

        submit(&store, &id, &transport).unwrap();

        assert_eq!(
            *transport.mottos.borrow(),
            ["Persisted manual motto".to_owned()]
        );
    }

    struct MottoTransport {
        mottos: RefCell<Vec<String>>,
    }

    impl ReportTransport for MottoTransport {
        fn deliver(&self, request: Url) -> DeliveryOutcome {
            self.mottos.borrow_mut().push(
                request
                    .query_pairs()
                    .find(|(key, _)| key == "m")
                    .unwrap()
                    .1
                    .into_owned(),
            );
            DeliveryOutcome::Delivered
        }
    }

    struct ProfileTransport {
        requests: RefCell<Vec<(String, String)>>,
        outcome: DeliveryOutcome,
    }

    impl ReportTransport for ProfileTransport {
        fn deliver(&self, request: Url) -> DeliveryOutcome {
            let trigger = request
                .query_pairs()
                .find(|(key, _)| key == "t")
                .unwrap()
                .1
                .into_owned();
            let motto = request
                .query_pairs()
                .find(|(key, _)| key == "m")
                .unwrap()
                .1
                .into_owned();
            self.requests.borrow_mut().push((trigger, motto));
            self.outcome
        }
    }

    struct GuildRecorder {
        calls: RefCell<Vec<(String, String)>>,
        outcome: GuildOutcome,
    }

    impl GuildTransport for GuildRecorder {
        fn guild(
            &self,
            request: Url,
            prior: &str,
            submitted: &str,
            _: &GuildResponseRules,
        ) -> GuildOutcome {
            assert_eq!(request.scheme(), "https");
            assert_eq!(request.host_str(), Some("progressquest.com"));
            assert_eq!(request.path(), "/alpaquil.php");
            let fields: std::collections::HashMap<_, _> = request.query_pairs().collect();
            assert_eq!(fields["cmd"], "guild");
            assert_eq!(fields["guild"], submitted);
            assert!(fields.contains_key("p"));
            assert!(!fields.contains_key("t"));
            self.calls
                .borrow_mut()
                .push((prior.to_owned(), submitted.to_owned()));
            self.outcome
        }
    }

    #[test]
    fn profile_persistence_is_visible_before_motto_delivery_and_after_guild_acceptance() {
        struct ObservingTransport {
            root: PathBuf,
            id: CharacterId,
        }
        impl ReportTransport for ObservingTransport {
            fn deliver(&self, request: Url) -> DeliveryOutcome {
                let profile = Store::open_at(&self.root)
                    .unwrap()
                    .profile(&self.id)
                    .unwrap();
                let motto = request
                    .query_pairs()
                    .find(|(key, _)| key == "m")
                    .unwrap()
                    .1
                    .into_owned();
                assert_eq!(profile.motto, motto);
                DeliveryOutcome::Delivered
            }
        }
        impl GuildTransport for ObservingTransport {
            fn guild(
                &self,
                _: Url,
                prior: &str,
                submitted: &str,
                _: &GuildResponseRules,
            ) -> GuildOutcome {
                let profile = Store::open_at(&self.root)
                    .unwrap()
                    .profile(&self.id)
                    .unwrap();
                assert_eq!(profile.guild, prior);
                assert_ne!(profile.guild, submitted);
                GuildOutcome::Accepted
            }
        }
        let (directory, mut store, id) = registered_store();
        let transport = ObservingTransport {
            root: directory.0.clone(),
            id: id.clone(),
        };
        set_motto(&mut store, &id, "Persisted first", &transport).unwrap();
        set_guild(&mut store, &id, "Accepted later", &transport).unwrap();
        assert_eq!(store.profile(&id).unwrap().guild, "Accepted later");
    }

    #[test]
    fn guild_join_change_and_leave_work_with_active_or_inactive_worker() {
        for active in [false, true] {
            let (directory, mut store, id) = registered_store();
            let worker = active
                .then(|| Worker::start(Store::open_at(&directory.0).unwrap(), id.clone()).unwrap());
            let original = store.get(&id).unwrap().state;
            let transport = GuildRecorder {
                calls: RefCell::new(Vec::new()),
                outcome: GuildOutcome::Accepted,
            };
            for designation in ["Test Guild", "\u{03b1} Guild", ""] {
                let prior = store.profile(&id).unwrap();
                assert_eq!(
                    set_guild(&mut store, &id, designation, &transport)
                        .unwrap()
                        .outcome,
                    GuildOutcome::Accepted
                );
                assert_eq!(store.profile(&id).unwrap().guild, designation);
                assert_eq!(store.profile(&id).unwrap().motto, prior.motto);
                assert_eq!(
                    serde_json::to_value(store.get(&id).unwrap().state.activity).unwrap(),
                    serde_json::to_value(&original.activity).unwrap()
                );
            }
            assert_eq!(
                *transport.calls.borrow(),
                [
                    (original.profile.guild, "Test Guild".into()),
                    ("Test Guild".into(), "\u{03b1} Guild".into()),
                    ("\u{03b1} Guild".into(), "".into()),
                ]
            );
            drop(worker);
        }
    }

    #[test]
    fn guild_failures_retain_membership_without_retry() {
        for outcome in [GuildOutcome::Rejected, GuildOutcome::Indeterminate] {
            for designation in ["New Guild", ""] {
                let (_directory, mut store, id) = registered_store();
                let profile = crate::state::OnlineProfile {
                    motto: "Motto".into(),
                    guild: "Prior".into(),
                };
                store.replace_profile(&id, &profile).unwrap();
                let transport = GuildRecorder {
                    calls: RefCell::new(Vec::new()),
                    outcome,
                };
                assert_eq!(
                    set_guild(&mut store, &id, designation, &transport)
                        .unwrap()
                        .outcome,
                    outcome
                );
                assert_eq!(store.profile(&id).unwrap(), profile);
                assert_eq!(transport.calls.borrow().len(), 1);
            }
        }
    }

    #[test]
    fn guild_evidence_and_text_fail_before_transport_or_mutation() {
        use crate::fixtures::GuildEvidenceError::*;
        let (_directory, mut store, id) = registered_store();
        let prior = store.profile(&id).unwrap();
        let transport = GuildRecorder {
            calls: RefCell::new(Vec::new()),
            outcome: GuildOutcome::Accepted,
        };
        for error in [
            Unavailable,
            Malformed,
            Sensitive,
            NotLiveOrPassing,
            Incomplete,
        ] {
            assert!(matches!(
                set_guild_with_evidence(&mut store, &id, "Test", &transport, Err(error)),
                Err(ReportingError::GuildEvidence(_))
            ));
        }
        assert!(matches!(
            set_guild(&mut store, &id, "bad\nvalue", &transport),
            Err(ReportingError::InvalidProfileText)
        ));
        assert!(transport.calls.borrow().is_empty());
        assert_eq!(store.profile(&id).unwrap(), prior);
    }

    #[test]
    fn motto_set_and_clear_persist_before_one_delivery() {
        let (_directory, mut store, id) = registered_store();
        let transport = ProfileTransport {
            requests: RefCell::new(Vec::new()),
            outcome: DeliveryOutcome::Delivered,
        };

        set_motto(&mut store, &id, "Progress safely", &transport).unwrap();
        assert_eq!(store.profile(&id).unwrap().motto, "Progress safely");
        set_motto(&mut store, &id, "", &transport).unwrap();
        assert_eq!(store.profile(&id).unwrap().motto, "");
        assert_eq!(
            *transport.requests.borrow(),
            [
                ("m".to_owned(), "Progress safely".to_owned()),
                ("m".to_owned(), String::new())
            ]
        );
    }

    #[test]
    fn motto_delivery_failure_retains_the_selected_value() {
        for outcome in [
            DeliveryOutcome::EndpointRejected,
            DeliveryOutcome::DeliveryFailed,
        ] {
            let (_directory, mut store, id) = registered_store();
            let transport = ProfileTransport {
                requests: RefCell::new(Vec::new()),
                outcome,
            };

            assert_eq!(
                set_motto(&mut store, &id, "Retained motto", &transport)
                    .unwrap()
                    .outcome,
                outcome
            );
            assert_eq!(store.profile(&id).unwrap().motto, "Retained motto");
            assert_eq!(transport.requests.borrow().len(), 1);
        }
    }

    #[test]
    fn motto_action_is_available_while_the_worker_is_active() {
        let (directory, mut store, id) = registered_store();
        let worker = Worker::start(Store::open_at(&directory.0).unwrap(), id.clone()).unwrap();
        let transport = ProfileTransport {
            requests: RefCell::new(Vec::new()),
            outcome: DeliveryOutcome::Delivered,
        };

        set_motto(&mut store, &id, "Live edit", &transport).unwrap();

        assert_eq!(store.profile(&id).unwrap().motto, "Live edit");
        assert_eq!(transport.requests.borrow().len(), 1);
        drop(worker);
    }

    #[test]
    fn motto_validation_rejects_before_persistence_or_transport() {
        let (_directory, mut store, id) = registered_store();
        let original = store.profile(&id).unwrap();
        let transport = ProfileTransport {
            requests: RefCell::new(Vec::new()),
            outcome: DeliveryOutcome::Delivered,
        };

        assert!(matches!(
            set_motto(&mut store, &id, "bad\nmotto", &transport),
            Err(ReportingError::InvalidProfileText)
        ));
        assert!(matches!(
            set_motto_with_evidence(
                &mut store,
                &id,
                "blocked",
                &transport,
                Err(EnrollmentEvidenceError::Unavailable)
            ),
            Err(ReportingError::Evidence(
                EnrollmentEvidenceError::Unavailable
            ))
        ));
        assert_eq!(store.profile(&id).unwrap(), original);
        assert!(transport.requests.borrow().is_empty());
    }

    #[test]
    fn motto_rejects_ineligible_credentials_and_endpoints_without_transport() {
        let directory = TestDirectory::new();
        let mut store = Store::open_at(&directory.0).unwrap();
        let transport = ProfileTransport {
            requests: RefCell::new(Vec::new()),
            outcome: DeliveryOutcome::Delivered,
        };
        let guild_transport = GuildRecorder {
            calls: RefCell::new(Vec::new()),
            outcome: GuildOutcome::Accepted,
        };

        let mut offline = save::import_text(
            &STANDARD.encode(include_str!("../tests/fixtures/reference-save.json")),
        )
        .unwrap();
        offline.online = None;
        let offline = store.register(&offline).unwrap();
        assert!(matches!(
            set_motto(&mut store, &offline.id, "Nope", &transport),
            Err(ReportingError::Storage(StorageError::ReportingIneligible))
        ));
        assert!(matches!(
            set_guild(&mut store, &offline.id, "Nope", &guild_transport),
            Err(ReportingError::Storage(StorageError::ReportingIneligible))
        ));

        let mut unofficial = save::import_text(
            &STANDARD.encode(include_str!("../tests/fixtures/reference-save.json")),
        )
        .unwrap();
        unofficial.online.as_mut().unwrap().host = "https://example.invalid/?".to_owned();
        let unofficial = store.register(&unofficial).unwrap();
        assert!(matches!(
            set_motto(&mut store, &unofficial.id, "Nope", &transport),
            Err(ReportingError::UnofficialEndpoint)
        ));
        assert!(matches!(
            set_guild(&mut store, &unofficial.id, "Nope", &guild_transport),
            Err(ReportingError::UnofficialEndpoint)
        ));

        let mut invalid_credential = save::import_text(
            &STANDARD.encode(include_str!("../tests/fixtures/reference-save.json")),
        )
        .unwrap();
        invalid_credential.document["online"]["passkey"] = serde_json::Value::String("bad".into());
        let invalid_credential = store.register(&invalid_credential).unwrap();
        assert!(matches!(
            set_motto(&mut store, &invalid_credential.id, "Nope", &transport),
            Err(ReportingError::Storage(StorageError::ReportingIneligible))
        ));
        assert!(matches!(
            set_guild(&mut store, &invalid_credential.id, "Nope", &guild_transport),
            Err(ReportingError::Storage(StorageError::ReportingIneligible))
        ));

        assert!(transport.requests.borrow().is_empty());
        assert!(guild_transport.calls.borrow().is_empty());
    }

    #[test]
    fn delivers_persisted_level_and_act_events_in_browser_order() {
        let (_directory, mut store, id) = registered_store();
        store
            .replace_profile(
                &id,
                &crate::state::OnlineProfile {
                    motto: "Persisted worker motto".to_owned(),
                    guild: String::new(),
                },
            )
            .unwrap();
        let mut initial = checkpoint::load(Path::new("tests/fixtures/checkpoint-level-up.json"))
            .unwrap()
            .initial;
        initial.online = store.get(&id).unwrap().state.online;
        initial.queue = vec!["plot|1|Loading".to_owned()];
        initial.stats.best = "STR".to_owned();
        initial.beststat = "STR 1".to_owned();
        let trace = advance_with_trace(
            &initial,
            &crate::ruleset::BUNDLED,
            1_000,
            "Stale event motto",
        )
        .unwrap();
        store.replace_state(&id, &trace.state).unwrap();
        let events = trace.events;
        assert_eq!(
            events.iter().map(|event| event.trigger).collect::<Vec<_>>(),
            vec![ReportTrigger::LevelUp, ReportTrigger::ActCompletion]
        );
        for event in &events {
            assert_eq!(event.snapshot.character.stats.best, "CHA");
            assert_eq!(event.snapshot.character.beststat, "CHA 16");
        }

        struct EventTransport {
            triggers: RefCell<Vec<String>>,
            mottos: RefCell<Vec<String>>,
            outcome: DeliveryOutcome,
        }
        impl ReportTransport for EventTransport {
            fn deliver(&self, request: Url) -> DeliveryOutcome {
                assert_progress_request(&request, 4242);
                assert_eq!(
                    request.query_pairs().find(|(key, _)| key == "k").unwrap().1,
                    "CHA 16"
                );
                self.triggers.borrow_mut().push(
                    request
                        .query_pairs()
                        .find(|(key, _)| key == "t")
                        .unwrap()
                        .1
                        .into_owned(),
                );
                self.mottos.borrow_mut().push(
                    request
                        .query_pairs()
                        .find(|(key, _)| key == "m")
                        .unwrap()
                        .1
                        .into_owned(),
                );
                self.outcome
            }
        }

        for outcome in [
            DeliveryOutcome::Delivered,
            DeliveryOutcome::EndpointRejected,
            DeliveryOutcome::DeliveryFailed,
        ] {
            let transport = EventTransport {
                triggers: RefCell::new(Vec::new()),
                mottos: RefCell::new(Vec::new()),
                outcome,
            };
            for event in &events {
                assert_eq!(
                    submit_event(&store, &id, event, &transport)
                        .unwrap()
                        .outcome,
                    outcome
                );
            }
            assert_eq!(*transport.triggers.borrow(), ["l", "a"]);
            assert_eq!(
                *transport.mottos.borrow(),
                [
                    "Persisted worker motto".to_owned(),
                    "Persisted worker motto".to_owned()
                ]
            );
        }

        let transport = EventTransport {
            triggers: RefCell::new(Vec::new()),
            mottos: RefCell::new(Vec::new()),
            outcome: DeliveryOutcome::Delivered,
        };
        let mut unofficial = events[0].clone();
        unofficial.snapshot.character.online.as_mut().unwrap().host =
            "https://example.invalid/?".to_owned();
        assert!(matches!(
            submit_event(&store, &id, &unofficial, &transport),
            Err(ReportingError::UnofficialEndpoint)
        ));
        assert!(transport.triggers.borrow().is_empty());
    }
}
