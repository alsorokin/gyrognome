use std::{
    io::Read,
    path::Path,
    thread,
    time::{Duration, Instant},
};

use serde::Serialize;
use thiserror::Error;
use url::Url;

use crate::{
    compatibility::{DesktopCanonicalState, DesktopImportMetadata},
    desktop_profile::{cell_text, html_unescape},
    desktop_protocol::{DesktopAccountAuthentication, DesktopReportOperation, report},
    desktop_save::DesktopValidatedSave,
    save::{ImportedSave, import_supported_file},
};

const REALM: &str = crate::desktop_contract::PEMPTUS.realm;
const SAVED_ENDPOINT: &str = crate::desktop_contract::PEMPTUS.saved_endpoint;
const ENDPOINT: &str = crate::desktop_contract::PEMPTUS.https_endpoint;
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
const OBSERVATION_BOUND: Duration = Duration::from_secs(60);
const POLL_INTERVAL: Duration = Duration::from_secs(5);
const MAX_REPORT_BYTES: usize = 16 * 1024;
const MAX_PAGE_BYTES: usize = 256 * 1024;

pub struct ProbeOptions {
    pub confirm_disposable: bool,
    pub confirm_client_stopped: bool,
    pub confirm_live_submission: bool,
    pub validate_only: bool,
}

#[derive(Debug, Error)]
pub enum ProbeError {
    #[error("probe requires explicit disposable-character and stopped-client confirmations")]
    MissingHandoff,
    #[error("probe requires explicit authorization for one live manual report")]
    MissingLiveApproval,
    #[error("probe input is not a supported desktop save")]
    UnsupportedSave,
    #[error("probe requires a canonical level-1 Pemptus passkey-only disposable import")]
    ContractMismatch,
    #[error("probe request construction or ASCII validation failed")]
    Construction,
    #[error("public baseline observation failed; no mutation attempted")]
    BaselineFailed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Delivery {
    NotAttempted,
    Delivered,
    EndpointRejected,
    RedirectRejected,
    NetworkFailure,
    ResponseTooLarge,
    InvalidPublicPage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Classification {
    Normal,
    Cheater,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ResponseCategory {
    Empty,
    ExplicitRejection,
    Unverified,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProbeResult {
    pub validation_only: bool,
    pub mutation_attempts: u8,
    pub delivery: Delivery,
    pub http_status: Option<u16>,
    pub response_category: Option<ResponseCategory>,
    pub baseline_classification: Classification,
    pub classification: Classification,
    pub public_state_matches: Option<bool>,
    pub public_state_changed: Option<bool>,
    pub observation_failure: Option<Delivery>,
    pub observation_seconds: f64,
    pub acceptance_independently_established: bool,
    pub production_eligibility_enabled: bool,
}

struct PreparedProbe {
    name: String,
    query: String,
    expected_cells: Vec<String>,
}

struct Request {
    endpoint: Url,
    timeout: Duration,
    max_bytes: usize,
}

struct Response {
    status: u16,
    body: Vec<u8>,
}

trait Transport {
    fn get(&mut self, request: Request) -> Result<Response, Delivery>;
}

struct HttpsTransport;

impl Transport for HttpsTransport {
    fn get(&mut self, request: Request) -> Result<Response, Delivery> {
        let mut response = http_request(&request)
            .call()
            .map_err(|_| Delivery::NetworkFailure)?;
        let status = response.status().as_u16();
        let mut body = Vec::new();
        response
            .body_mut()
            .as_reader()
            .take(request.max_bytes as u64 + 1)
            .read_to_end(&mut body)
            .map_err(|_| Delivery::NetworkFailure)?;
        checked_response(Response { status, body }, request.max_bytes)
    }
}

fn http_request(request: &Request) -> ureq::RequestBuilder<ureq::typestate::WithoutBody> {
    // A fresh agent avoids pooled-connection recovery for the one-shot mutation.
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .https_only(true)
        .max_redirects(0)
        .http_status_as_error(false)
        .timeout_global(Some(request.timeout))
        .build()
        .into();
    agent.get(request.endpoint.as_str())
}

trait Clock {
    fn now(&self) -> Duration;
    fn sleep(&mut self, duration: Duration);
}

struct MonotonicClock(Instant);

impl Clock for MonotonicClock {
    fn now(&self) -> Duration {
        self.0.elapsed()
    }

    fn sleep(&mut self, duration: Duration) {
        thread::sleep(duration);
    }
}

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct Observation {
    pub(crate) classification: Classification,
    pub(crate) cells: Option<Vec<String>>,
}

pub fn run(path: &Path, options: &ProbeOptions) -> Result<ProbeResult, ProbeError> {
    validate_options(options)?;
    let ImportedSave::Desktop(save) =
        import_supported_file(path).map_err(|_| ProbeError::UnsupportedSave)?
    else {
        return Err(ProbeError::UnsupportedSave);
    };
    execute(
        &save,
        options,
        &mut HttpsTransport,
        &mut MonotonicClock(Instant::now()),
    )
}

fn validate_options(options: &ProbeOptions) -> Result<(), ProbeError> {
    if !options.confirm_disposable || !options.confirm_client_stopped {
        return Err(ProbeError::MissingHandoff);
    }
    if !options.validate_only && !options.confirm_live_submission {
        return Err(ProbeError::MissingLiveApproval);
    }
    Ok(())
}

fn prepare(save: &DesktopValidatedSave) -> Result<PreparedProbe, ProbeError> {
    let metadata = DesktopImportMetadata::from_validated(&save.adaptations);
    let level = save
        .traits
        .iter()
        .find(|row| row.caption == "Level")
        .and_then(|row| row.subitems.first());
    if save.private.realm != REALM
        || save.private.endpoint != SAVED_ENDPOINT
        || save.private.passkey <= 0
        || !save.private.account.is_empty()
        || !save.private.password.is_empty()
        || !metadata.provenance.adaptations.is_empty()
        || level.map(String::as_str) != Some("1")
        || !(1..=2).contains(&save.plots.len())
        || save.quests.len() > 1
    {
        return Err(ProbeError::ContractMismatch);
    }
    let state = DesktopCanonicalState::from(save);
    let request = report(
        &state,
        DesktopReportOperation::Manual,
        REALM,
        &save.profile.motto,
        save.private.passkey,
        &DesktopAccountAuthentication::new("", ""),
    )
    .map_err(|_| ProbeError::Construction)?;
    let expected_cells = request.public_cells().ok_or(ProbeError::Construction)?;
    Ok(PreparedProbe {
        name: expected_cells[0].clone(),
        query: request.encoded_query(),
        expected_cells,
    })
}

fn execute(
    save: &DesktopValidatedSave,
    options: &ProbeOptions,
    transport: &mut impl Transport,
    clock: &mut impl Clock,
) -> Result<ProbeResult, ProbeError> {
    validate_options(options)?;
    let prepared = prepare(save)?;
    let mut result = ProbeResult {
        validation_only: options.validate_only,
        mutation_attempts: 0,
        delivery: Delivery::NotAttempted,
        http_status: None,
        response_category: None,
        baseline_classification: Classification::Unknown,
        classification: Classification::Unknown,
        public_state_matches: None,
        public_state_changed: None,
        observation_failure: None,
        observation_seconds: 0.0,
        acceptance_independently_established: false,
        production_eligibility_enabled: false,
    };
    if options.validate_only {
        return Ok(result);
    }
    let baseline = observe(transport, &prepared.name, REQUEST_TIMEOUT)
        .map_err(|_| ProbeError::BaselineFailed)?;
    result.baseline_classification = baseline.classification;
    let mut endpoint = fixed_endpoint();
    endpoint.set_query(Some(&prepared.query));
    result.mutation_attempts = 1;
    match transport
        .get(Request {
            endpoint,
            timeout: REQUEST_TIMEOUT,
            max_bytes: MAX_REPORT_BYTES,
        })
        .and_then(|response| checked_response(response, MAX_REPORT_BYTES))
    {
        Ok(response) => {
            result.delivery = Delivery::Delivered;
            result.http_status = Some(response.status);
            result.response_category = Some(response_category(&response.body));
        }
        Err(error) => result.delivery = error,
    }
    let start = clock.now();
    let deadline = start + OBSERVATION_BOUND;
    while clock.now() < deadline {
        let remaining = deadline.saturating_sub(clock.now());
        if remaining.is_zero() {
            break;
        }
        match observe(transport, &prepared.name, REQUEST_TIMEOUT.min(remaining)) {
            Ok(observation) if clock.now() <= deadline => {
                result.observation_failure = None;
                result.classification = observation.classification;
                result.public_state_matches = observation
                    .cells
                    .as_ref()
                    .map(|cells| cells == &prepared.expected_cells);
                result.public_state_changed = observation
                    .cells
                    .as_ref()
                    .map(|cells| baseline.cells.as_ref() != Some(cells));
                if result.classification == Classification::Cheater
                    || (result.classification == Classification::Normal
                        && result.public_state_matches == Some(true))
                {
                    break;
                }
            }
            Ok(_) => {
                result.classification = Classification::Unknown;
                result.public_state_matches = None;
                result.public_state_changed = None;
            }
            Err(error) => {
                result.observation_failure = Some(error);
                result.classification = Classification::Unknown;
                result.public_state_matches = None;
                result.public_state_changed = None;
            }
        }
        clock.sleep(POLL_INTERVAL.min(deadline.saturating_sub(clock.now())));
    }
    result.observation_seconds = clock.now().saturating_sub(start).as_secs_f64();
    // Without a separately evidenced response contract, public changes are corroboration only.
    Ok(result)
}

fn fixed_endpoint() -> Url {
    Url::parse(ENDPOINT).expect("fixed Pemptus HTTPS endpoint")
}

fn checked_response(response: Response, max_bytes: usize) -> Result<Response, Delivery> {
    if (300..400).contains(&response.status) {
        return Err(Delivery::RedirectRejected);
    }
    if !(200..300).contains(&response.status) {
        return Err(Delivery::EndpointRejected);
    }
    if response.body.len() > max_bytes {
        return Err(Delivery::ResponseTooLarge);
    }
    Ok(response)
}

fn response_category(body: &[u8]) -> ResponseCategory {
    if body.is_empty() {
        return ResponseCategory::Empty;
    }
    let Ok(text) = std::str::from_utf8(body) else {
        return ResponseCategory::Unverified;
    };
    match text.split('|').next().unwrap_or_default().trim() {
        "error" | "rejected" | "denied" => ResponseCategory::ExplicitRejection,
        _ => ResponseCategory::Unverified,
    }
}

fn observe(
    transport: &mut impl Transport,
    name: &str,
    timeout: Duration,
) -> Result<Observation, Delivery> {
    let mut endpoint = fixed_endpoint();
    endpoint.query_pairs_mut().append_pair("name", name);
    let response = transport
        .get(Request {
            endpoint,
            timeout,
            max_bytes: MAX_PAGE_BYTES,
        })
        .and_then(|response| checked_response(response, MAX_PAGE_BYTES))?;
    let page = std::str::from_utf8(&response.body).map_err(|_| Delivery::InvalidPublicPage)?;
    parse_observation(page, name).ok_or(Delivery::InvalidPublicPage)
}

pub(crate) fn parse_observation(page: &str, name: &str) -> Option<Observation> {
    let mut found = None;
    let mut offset = 0;
    while let Some(start) = page[offset..].find("<table") {
        let start = offset + start;
        let content = start + page[start..].find('>')? + 1;
        let end = content + page[content..].find("</table>")?;
        let prefix = &page[..start];
        let heading_start = prefix.rfind("<h1>")? + "<h1>".len();
        let heading_end = heading_start + prefix[heading_start..].find("</h1>")?;
        let classification = match prefix[heading_start..heading_end].trim() {
            "Hall of Fame" => Classification::Normal,
            "Hall of Infamy" => Classification::Cheater,
            _ => Classification::Unknown,
        };
        for row in page[content..end].split("<tr").skip(1) {
            if !row.starts_with('>') && !row.starts_with(char::is_whitespace) {
                return None;
            }
            let cells = row
                .split("<td")
                .skip(1)
                .map(|cell| {
                    let (attributes, text) = cell.split_once('>')?;
                    if !attributes.is_empty() && !attributes.starts_with(char::is_whitespace) {
                        return None;
                    }
                    html_unescape(cell_text(text)?.trim())
                })
                .collect::<Option<Vec<_>>>()?;
            if cells.get(1).map(String::as_str) != Some(name) {
                continue;
            }
            if !(10..=11).contains(&cells.len()) || found.is_some() {
                return None;
            }
            found = Some(Observation {
                classification,
                cells: Some(cells[1..10].to_vec()),
            });
        }
        offset = end + "</table>".len();
    }
    if offset == 0 {
        return None;
    }
    Some(found.unwrap_or(Observation {
        classification: Classification::Unknown,
        cells: None,
    }))
}

#[cfg(test)]
pub(crate) mod tests {
    use std::{cell::Cell, collections::VecDeque, rc::Rc};

    use super::*;
    use crate::desktop_save::{
        DesktopAdaptations, DesktopQuestMarker, DesktopValidatedBar, DesktopValidatedBars,
        DesktopValidatedPrivateMetadata, DesktopValidatedProfile, DesktopValidatedRow,
    };

    fn row(caption: &str, value: &str) -> DesktopValidatedRow {
        DesktopValidatedRow {
            header: [0, -1, -1, 1, 0],
            caption: caption.to_owned(),
            subitems: vec![value.to_owned()],
        }
    }

    pub(crate) fn synthetic_save() -> DesktopValidatedSave {
        let bar = DesktopValidatedBar {
            position: 0,
            maximum: 100,
        };
        DesktopValidatedSave {
            traits: vec![
                row("Name", "Synthetic Hero"),
                row("Race", "Half Orc"),
                row("Class", "Robot Monk"),
                row("Level", "1"),
            ],
            stats: ["STR", "CON", "DEX", "INT", "WIS", "CHA"]
                .iter()
                .map(|caption| row(caption, "12"))
                .collect(),
            equipment: vec![row("Weapon", "Sharp Stick")],
            inventory: vec![row("Gold", "0")],
            spells: Vec::new(),
            plots: vec![row("Prologue", ""), row("Act I", "")],
            quests: Vec::new(),
            current_task: "load".to_owned(),
            quest: DesktopQuestMarker::None,
            queue: Vec::new(),
            activity: "Loading".to_owned(),
            bars: DesktopValidatedBars {
                experience: DesktopValidatedBar {
                    position: 42,
                    maximum: 1269,
                },
                encumbrance: bar.clone(),
                plot: bar.clone(),
                quest: bar.clone(),
                task: bar,
            },
            prized_equipment: 0,
            game_style: 3,
            profile: DesktopValidatedProfile {
                motto: "Ready & waiting".to_owned(),
                guild: String::new(),
            },
            private: DesktopValidatedPrivateMetadata {
                passkey: 42_424,
                realm: REALM.to_owned(),
                endpoint: SAVED_ENDPOINT.to_owned(),
                account: String::new(),
                password: String::new(),
            },
            adaptations: DesktopAdaptations {
                legacy_prologue_62: false,
                legacy_quest_placeholder: false,
                spelling_patch_applied: false,
            },
            restored: None,
        }
    }

    fn options() -> ProbeOptions {
        ProbeOptions {
            confirm_disposable: true,
            confirm_client_stopped: true,
            confirm_live_submission: true,
            validate_only: false,
        }
    }

    fn page(cells: &[String], heading: &str) -> String {
        let escape = |text: &str| text.replace('&', "&amp;").replace('<', "&lt;");
        format!(
            "<h1>{heading}</h1><table><tr><th>Name<tr class=selected><td>1{}</table>",
            cells
                .iter()
                .map(|cell| format!("<td>{}", escape(cell)))
                .collect::<String>()
        )
    }

    fn response(body: impl Into<Vec<u8>>) -> Result<Response, Delivery> {
        Ok(Response {
            status: 200,
            body: body.into(),
        })
    }

    struct FakeClock(Rc<Cell<Duration>>);

    impl Clock for FakeClock {
        fn now(&self) -> Duration {
            self.0.get()
        }

        fn sleep(&mut self, duration: Duration) {
            self.0.set(self.now() + duration);
        }
    }

    struct FakeTransport {
        responses: VecDeque<Result<Response, Delivery>>,
        mutations: usize,
        public_reads: usize,
        timeouts: Vec<Duration>,
        time: Rc<Cell<Duration>>,
        consume_timeout: bool,
    }

    impl Transport for FakeTransport {
        fn get(&mut self, request: Request) -> Result<Response, Delivery> {
            assert_eq!(request.endpoint.scheme(), "https");
            assert_eq!(request.endpoint.host_str(), Some("progressquest.com"));
            assert_eq!(request.endpoint.path(), "/pemptus.php");
            assert!(request.endpoint.username().is_empty());
            assert!(request.endpoint.password().is_none());
            let query = request.endpoint.query_pairs().collect::<Vec<_>>();
            if query.iter().any(|(key, _)| key == "cmd") {
                self.mutations += 1;
                assert_eq!(request.max_bytes, MAX_REPORT_BYTES);
                assert!(query.iter().any(|(key, value)| key == "t" && value == "b"));
            } else {
                self.public_reads += 1;
                assert_eq!(query.len(), 1);
                assert_eq!(query[0].0, "name");
                assert_eq!(request.max_bytes, MAX_PAGE_BYTES);
            }
            assert!(
                http_request(&request)
                    .headers_ref()
                    .unwrap()
                    .get("Authorization")
                    .is_none()
            );
            self.timeouts.push(request.timeout);
            if self.consume_timeout {
                self.time.set(self.time.get() + request.timeout);
            }
            self.responses
                .pop_front()
                .unwrap_or(Err(Delivery::NetworkFailure))
        }
    }

    fn fake(responses: Vec<Result<Response, Delivery>>) -> (FakeTransport, FakeClock) {
        let time = Rc::new(Cell::new(Duration::ZERO));
        (
            FakeTransport {
                responses: responses.into(),
                mutations: 0,
                public_reads: 0,
                timeouts: Vec::new(),
                time: time.clone(),
                consume_timeout: false,
            },
            FakeClock(time),
        )
    }

    #[test]
    fn validates_all_preconditions_without_networking() {
        let mutations: [fn(&mut DesktopValidatedSave); 14] = [
            |save| save.private.realm = "Spoltog".to_owned(),
            |save| save.private.endpoint = "https://example.invalid/".to_owned(),
            |save| save.private.passkey = 0,
            |save| save.private.account = "synthetic".to_owned(),
            |save| save.private.password = "synthetic".to_owned(),
            |save| save.adaptations.legacy_prologue_62 = true,
            |save| save.adaptations.legacy_quest_placeholder = true,
            |save| save.adaptations.spelling_patch_applied = true,
            |save| save.traits[3].subitems[0] = "2".to_owned(),
            |save| save.plots.clear(),
            |save| save.plots.push(row("Act II", "")),
            |save| save.quests = vec![row("First", ""), row("Second", "")],
            |save| save.profile.motto = "\u{00e9}".to_owned(),
            |save| save.traits[0].subitems[0] = "\u{00e9}".to_owned(),
        ];
        for mutate in mutations {
            let mut save = synthetic_save();
            mutate(&mut save);
            let (mut transport, mut clock) = fake(vec![]);
            assert!(execute(&save, &options(), &mut transport, &mut clock).is_err());
            assert_eq!(transport.public_reads + transport.mutations, 0);
        }
        for i in 0..3 {
            let mut options = options();
            match i {
                0 => options.confirm_disposable = false,
                1 => options.confirm_client_stopped = false,
                _ => options.confirm_live_submission = false,
            }
            let (mut transport, mut clock) = fake(vec![]);
            assert!(execute(&synthetic_save(), &options, &mut transport, &mut clock).is_err());
            assert_eq!(transport.public_reads + transport.mutations, 0);
        }
        let mut options = options();
        options.validate_only = true;
        options.confirm_live_submission = false;
        let (mut transport, mut clock) = fake(vec![]);
        let result = execute(&synthetic_save(), &options, &mut transport, &mut clock).unwrap();
        assert!(result.validation_only);
        assert_eq!(transport.public_reads + transport.mutations, 0);
    }

    #[test]
    fn native_manual_query_matches_source_derived_synthetic_validator() {
        let save = synthetic_save();
        let prepared = prepare(&save).unwrap();
        assert_eq!(
            prepared.query,
            "cmd=b&t=b&n=Synthetic+Hero&r=Half+Orc&c=Robot+Monk&l=1&x=42&i=Sharp+Stick&k=STR+12&a=Act+I&h=Pemptus&rev=8&p=369908475&m=Ready+%26+waiting"
        );
        assert!(!prepared.query.contains("&z="));
        assert_eq!(prepared.expected_cells[4], "STR 12");
        assert_eq!(prepared.expected_cells[8], "Ready & waiting");
    }

    #[test]
    fn probe_does_not_enable_pemptus_production_reporting() {
        use crate::desktop_eligibility::{DesktopEligibilityDecision, DesktopIneligibilityReason};
        crate::desktop_evidence::with_synthetic_desktop_evidence(vec![], || {
            let inspection = crate::save::inspect_desktop(&synthetic_save());
            assert_eq!(inspection.online_eligibility.len(), 5);
            assert!(inspection.online_eligibility.iter().all(|operation| {
                operation.decision
                    == DesktopEligibilityDecision::Ineligible(
                        DesktopIneligibilityReason::OperationEvidenceUnavailable,
                    )
            }));
        });
    }

    #[test]
    fn exact_row_identity_and_classification_reject_ambiguous_markup() {
        let cells = prepare(&synthetic_save()).unwrap().expected_cells;
        let normal = page(&cells, "Hall of Fame");
        let name = &cells[0];
        assert_eq!(
            parse_observation(&normal, name).unwrap().classification,
            Classification::Normal
        );
        let cheater = page(&cells, "Hall of Infamy");
        assert_eq!(
            parse_observation(&cheater, name).unwrap().classification,
            Classification::Cheater
        );
        assert_eq!(
            parse_observation(&normal, "Synthetic")
                .unwrap()
                .classification,
            Classification::Unknown
        );
        assert_eq!(
            parse_observation(&format!("<title>{name}</title>{normal}"), "Other")
                .unwrap()
                .classification,
            Classification::Unknown
        );
        assert!(parse_observation(&normal.replace("</table>", ""), name).is_none());
        assert!(parse_observation(&format!("{normal}{normal}"), name).is_none());
        assert!(parse_observation("<h1>Hall of Fame</h1>no table", name).is_none());
        assert_eq!(
            parse_observation(&normal.replace("Hall of Fame", "Other"), name)
                .unwrap()
                .classification,
            Classification::Unknown
        );
        let anchored = normal.replace("<td>Synthetic Hero", "<td><a href='/'>Synthetic Hero</a>");
        assert_eq!(
            parse_observation(&anchored, name).unwrap().cells,
            Some(cells)
        );
    }

    #[test]
    fn unchanged_matching_normal_row_is_not_acceptance() {
        let save = synthetic_save();
        let original = save.clone();
        let normal = page(&prepare(&save).unwrap().expected_cells, "Hall of Fame");
        let (mut transport, mut clock) = fake(vec![
            response(normal.clone()),
            response(Vec::new()),
            response(normal),
        ]);
        let result = execute(&save, &options(), &mut transport, &mut clock).unwrap();
        assert_eq!(result.delivery, Delivery::Delivered);
        assert_eq!(result.classification, Classification::Normal);
        assert_eq!(result.public_state_matches, Some(true));
        assert_eq!(result.public_state_changed, Some(false));
        assert!(!result.acceptance_independently_established);
        assert!(!result.production_eligibility_enabled);
        assert_eq!(transport.mutations, 1);
        assert_eq!(save, original);
        let output = serde_json::to_string(&result).unwrap();
        for secret in [
            "42424",
            "369908475",
            "Synthetic Hero",
            "Ready & waiting",
            "cmd=b",
        ] {
            assert!(!output.contains(secret));
        }
    }

    #[test]
    fn changed_report_visible_state_is_reported_separately() {
        let save = synthetic_save();
        let expected = prepare(&save).unwrap().expected_cells;
        let mut baseline = expected.clone();
        baseline[4] = "DEX 11".to_owned();
        let (mut transport, mut clock) = fake(vec![
            response(page(&baseline, "Hall of Fame")),
            response(Vec::new()),
            response(page(&expected, "Hall of Fame")),
        ]);
        let result = execute(&save, &options(), &mut transport, &mut clock).unwrap();
        assert_eq!(result.public_state_changed, Some(true));
        assert_eq!(result.public_state_matches, Some(true));
        assert!(!result.acceptance_independently_established);
        assert_eq!(transport.mutations, 1);
    }

    #[test]
    fn mutation_failures_never_retry_or_become_acceptance() {
        let save = synthetic_save();
        let normal = page(&prepare(&save).unwrap().expected_cells, "Hall of Fame");
        let failures = vec![
            (Err(Delivery::NetworkFailure), Delivery::NetworkFailure),
            (
                response(vec![b'x'; MAX_REPORT_BYTES + 1]),
                Delivery::ResponseTooLarge,
            ),
            (
                Ok(Response {
                    status: 302,
                    body: vec![],
                }),
                Delivery::RedirectRejected,
            ),
            (
                Ok(Response {
                    status: 403,
                    body: vec![],
                }),
                Delivery::EndpointRejected,
            ),
        ];
        for (failure, expected_delivery) in failures {
            let (mut transport, mut clock) = fake(vec![
                response(normal.clone()),
                failure,
                response(normal.clone()),
            ]);
            let result = execute(&save, &options(), &mut transport, &mut clock).unwrap();
            assert_eq!(result.delivery, expected_delivery);
            assert_eq!(result.mutation_attempts, 1);
            assert_eq!(transport.mutations, 1);
            assert!(!result.acceptance_independently_established);
        }
        let (mut transport, mut clock) = fake(vec![
            response(normal.clone()),
            response(b"error|synthetic rejection".to_vec()),
            response(normal),
        ]);
        let result = execute(&save, &options(), &mut transport, &mut clock).unwrap();
        assert_eq!(
            result.response_category,
            Some(ResponseCategory::ExplicitRejection)
        );
        assert!(!result.acceptance_independently_established);
    }

    #[test]
    fn observation_deadline_clips_each_request_and_sleep() {
        let save = synthetic_save();
        let normal = page(&prepare(&save).unwrap().expected_cells, "Hall of Fame");
        let (mut transport, mut clock) = fake(vec![response(normal), response(vec![])]);
        transport.consume_timeout = true;
        let result = execute(&save, &options(), &mut transport, &mut clock).unwrap();
        assert_eq!(result.observation_seconds, 60.0);
        assert_eq!(
            transport.timeouts,
            vec![
                Duration::from_secs(30),
                Duration::from_secs(30),
                Duration::from_secs(30),
                Duration::from_secs(25),
            ]
        );
        assert_eq!(result.classification, Classification::Unknown);
        assert_eq!(result.observation_failure, Some(Delivery::NetworkFailure));
        assert_eq!(transport.mutations, 1);
    }

    #[test]
    fn invalid_baseline_aborts_before_mutation_and_cheater_is_not_normal() {
        let save = synthetic_save();
        for bad in [
            Err(Delivery::NetworkFailure),
            response(vec![b'x'; MAX_PAGE_BYTES + 1]),
            response(b"malformed page".to_vec()),
        ] {
            let (mut transport, mut clock) = fake(vec![bad]);
            assert!(matches!(
                execute(&save, &options(), &mut transport, &mut clock),
                Err(ProbeError::BaselineFailed)
            ));
            assert_eq!(transport.mutations, 0);
        }
        let cells = prepare(&save).unwrap().expected_cells;
        let (mut transport, mut clock) = fake(vec![
            response(page(&cells, "Hall of Fame")),
            response(vec![]),
            response(page(&cells, "Hall of Infamy")),
        ]);
        let result = execute(&save, &options(), &mut transport, &mut clock).unwrap();
        assert_eq!(result.classification, Classification::Cheater);
        assert!(!result.acceptance_independently_established);
    }

    #[test]
    fn unknown_or_malformed_public_rows_remain_inconclusive() {
        let save = synthetic_save();
        let cells = prepare(&save).unwrap().expected_cells;
        for observation in [
            response(page(&cells, "Other")),
            response(b"<table>bad".to_vec()),
            response(vec![b'x'; MAX_PAGE_BYTES + 1]),
            response(vec![0xff]),
        ] {
            let (mut transport, mut clock) = fake(vec![
                response(page(&cells, "Hall of Fame")),
                response(vec![]),
                observation,
            ]);
            let result = execute(&save, &options(), &mut transport, &mut clock).unwrap();
            assert_eq!(result.classification, Classification::Unknown);
            assert!(!result.acceptance_independently_established);
            assert_eq!(result.observation_seconds, 60.0);
            assert_eq!(transport.mutations, 1);
        }
    }
}
