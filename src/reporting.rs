//! Credential-safe leaderboard report delivery.

use thiserror::Error;
use url::Url;

use crate::{
    fixtures::{EnrollmentEvidenceError, validate_bundled_enrollment_evidence},
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
    Evidence(#[from] EnrollmentEvidenceError),
    #[error(transparent)]
    Storage(#[from] StorageError),
    #[error("managed character does not use the official leaderboard endpoint")]
    UnofficialEndpoint,
    #[error("could not construct the leaderboard report")]
    Construction,
    #[error("online enrollment is incomplete; no local character was registered")]
    IncompleteEnrollment,
}

pub trait ReportTransport: Send {
    fn deliver(&self, request: Url) -> DeliveryOutcome;
}

pub trait EnrollmentTransport: ReportTransport {
    fn create(&self, request: Url) -> CreateDelivery;
}

pub struct HttpsTransport;

impl ReportTransport for HttpsTransport {
    fn deliver(&self, request: Url) -> DeliveryOutcome {
        match ureq::get(request.as_str()).call() {
            Ok(response) if response.status().is_success() => DeliveryOutcome::Delivered,
            Ok(_) => DeliveryOutcome::EndpointRejected,
            Err(_) => DeliveryOutcome::DeliveryFailed,
        }
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
        self.report
    }
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
    {
        return Err(ReportingError::UnofficialEndpoint);
    }
    Ok(endpoint)
}

pub struct ReportResult {
    pub identity: CharacterIdentity,
    pub outcome: DeliveryOutcome,
}

pub fn submit(
    store: &Store,
    id: &CharacterId,
    transport: &impl ReportTransport,
) -> Result<ReportResult, ReportingError> {
    validate_bundled_enrollment_evidence()?;
    let target = store.reporting_target(id)?;
    let mut state = target.state;
    crate::simulation::update_bestspell(&mut state);
    let host = state
        .online
        .as_ref()
        .map(|online| online.host.as_str())
        .ok_or(StorageError::ReportingIneligible)?;
    let endpoint = official_endpoint(host)?;
    if endpoint.as_str().trim_end_matches('?') != OFFICIAL_LEADERBOARD_ENDPOINT {
        return Err(ReportingError::UnofficialEndpoint);
    }
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
            motto: "",
        },
        target.passkey,
    )
    .map_err(|_| ReportingError::Construction)?;
    let request = official_endpoint(&request)?;
    Ok(ReportResult {
        identity: target.identity,
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
    let endpoint = official_endpoint(host)?;
    if endpoint.as_str().trim_end_matches('?') != OFFICIAL_LEADERBOARD_ENDPOINT {
        return Err(ReportingError::UnofficialEndpoint);
    }
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
            motto: &event.motto,
        },
        target.passkey,
    )
    .map_err(|_| ReportingError::Construction)?;
    let request = official_endpoint(&request)?;
    Ok(ReportResult {
        identity: target.identity,
        outcome: transport.deliver(request),
    })
}

#[cfg(test)]
mod tests {
    use std::{
        cell::RefCell,
        fs,
        path::{Path, PathBuf},
    };

    use base64::{Engine, engine::general_purpose::STANDARD};
    use uuid::Uuid;

    use super::*;
    use crate::{
        checkpoint, newguy,
        runtime::Store,
        save,
        simulation::{ReportTrigger, advance_with_trace},
    };

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

    struct RecordingTransport {
        requests: RefCell<Vec<Vec<String>>>,
    }

    impl ReportTransport for RecordingTransport {
        fn deliver(&self, request: Url) -> DeliveryOutcome {
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
        let draft = newguy::generate(
            &newguy::Selection {
                name: "Online Hero".to_owned(),
                race: "Gyrognome".to_owned(),
                class: "Robot Monk".to_owned(),
            },
            &crate::ruleset::BUNDLED,
            &mut Numbers(1),
        )
        .unwrap();
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

    struct SpecialtyTransport {
        specialty: RefCell<Option<String>>,
    }

    impl ReportTransport for SpecialtyTransport {
        fn deliver(&self, request: Url) -> DeliveryOutcome {
            *self.specialty.borrow_mut() = request
                .query_pairs()
                .find(|(key, _)| key == "z")
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
    fn delivers_one_manual_brag_with_safe_outcomes() {
        for expected in [
            DeliveryOutcome::Delivered,
            DeliveryOutcome::EndpointRejected,
            DeliveryOutcome::DeliveryFailed,
        ] {
            let (_directory, store, id) = registered_store();
            let result = submit(
                &store,
                &id,
                &FakeTransport {
                    create: CreateDelivery::DeliveryFailed,
                    report: expected,
                },
            )
            .unwrap();
            assert_eq!(result.outcome, expected);
            assert_eq!(result.identity.name, "Reference Hero");
        }
    }

    #[test]
    fn manual_brag_derives_specialty_before_constructing_the_report() {
        let (_directory, mut store, id) = registered_store();
        let mut state = store.get(&id).unwrap().state;
        state.bestspell = "Stale Specialty".to_owned();
        store.replace_state(&id, &state).unwrap();
        let transport = SpecialtyTransport {
            specialty: RefCell::new(None),
        };

        submit(&store, &id, &transport).unwrap();

        assert_eq!(
            *transport.specialty.borrow(),
            Some("Hastiness II".to_owned())
        );
    }

    #[test]
    fn delivers_persisted_level_and_act_events_in_browser_order() {
        let (_directory, store, id) = registered_store();
        let mut initial = checkpoint::load(Path::new("tests/fixtures/checkpoint-level-up.json"))
            .unwrap()
            .initial;
        initial.online = store.get(&id).unwrap().state.online;
        initial.queue = vec!["plot|1|Loading".to_owned()];
        let events = advance_with_trace(
            &initial,
            &crate::ruleset::BUNDLED,
            1_000,
            "Credential-safe motto",
        )
        .unwrap()
        .events;
        assert_eq!(
            events.iter().map(|event| event.trigger).collect::<Vec<_>>(),
            vec![ReportTrigger::LevelUp, ReportTrigger::ActCompletion]
        );

        struct EventTransport {
            triggers: RefCell<Vec<String>>,
            outcome: DeliveryOutcome,
        }
        impl ReportTransport for EventTransport {
            fn deliver(&self, request: Url) -> DeliveryOutcome {
                assert_eq!(
                    request.as_str().split('?').next(),
                    Some(OFFICIAL_LEADERBOARD_ENDPOINT)
                );
                self.triggers.borrow_mut().push(
                    request
                        .query_pairs()
                        .find(|(key, _)| key == "t")
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
        }
    }
}
