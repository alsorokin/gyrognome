//! Explicit, foreground-only leaderboard report delivery.

use thiserror::Error;
use url::Url;

use crate::{
    fixtures::{EnrollmentEvidenceError, validate_bundled_enrollment_evidence},
    protocol::{self, ReportFields},
    runtime::{CharacterId, CharacterIdentity, StorageError, Store},
};

pub const OFFICIAL_LEADERBOARD_ENDPOINT: &str = "https://progressquest.com/alpaquil.php";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeliveryOutcome {
    Delivered,
    EndpointRejected,
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
}

pub trait ReportTransport {
    fn deliver(&self, request: Url) -> DeliveryOutcome;
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
    let host = target
        .state
        .online
        .as_ref()
        .map(|online| online.host.as_str())
        .ok_or(StorageError::ReportingIneligible)?;
    let endpoint = Url::parse(host).map_err(|_| ReportingError::UnofficialEndpoint)?;
    if endpoint.as_str().trim_end_matches('?') != OFFICIAL_LEADERBOARD_ENDPOINT {
        return Err(ReportingError::UnofficialEndpoint);
    }
    let request = protocol::progress_report(
        host,
        &target.state,
        'b',
        ReportFields {
            xp_position: target.state.progress.experience.position as u64,
            best_equipment: &target.state.bestequip,
            best_spell: &target.state.bestspell,
            best_stat: &target.state.beststat,
            best_plot: &target.state.plot.bestplot,
            motto: "",
        },
        target.passkey,
    )
    .map_err(|_| ReportingError::Construction)?;
    let request = Url::parse(&request).map_err(|_| ReportingError::Construction)?;
    Ok(ReportResult {
        identity: target.identity,
        outcome: transport.deliver(request),
    })
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::{Path, PathBuf},
    };

    use base64::{Engine, engine::general_purpose::STANDARD};
    use uuid::Uuid;

    use super::*;
    use crate::{runtime::Store, save};

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
    struct FakeTransport(DeliveryOutcome);
    impl ReportTransport for FakeTransport {
        fn deliver(&self, request: Url) -> DeliveryOutcome {
            assert_eq!(request.scheme(), "https");
            assert_eq!(request.host_str(), Some("progressquest.com"));
            assert_eq!(request.path(), "/alpaquil.php");
            assert_eq!(
                request.query_pairs().find(|(key, _)| key == "t").unwrap().1,
                "b"
            );
            self.0
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
            let result = submit(&store, &id, &FakeTransport(expected)).unwrap();
            assert_eq!(result.outcome, expected);
            assert_eq!(result.identity.name, "Reference Hero");
        }
    }
}
