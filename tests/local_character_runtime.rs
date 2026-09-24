use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::Duration,
};

use base64::{Engine, engine::general_purpose::STANDARD};
use gyrognome::{
    dashboard,
    runtime::{CharacterId, Store, Worker},
    save,
};
use uuid::Uuid;

struct TestDirectory(PathBuf);

impl TestDirectory {
    fn new(label: &str) -> Self {
        let path = Path::new("target")
            .join("gyrognome-runtime-integration-tests")
            .join(format!("{label}-{}", Uuid::new_v4()));
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }

    fn save_file(&self) -> PathBuf {
        let path = self.0.join("synthetic.pqw");
        let encoded = STANDARD.encode(include_str!("fixtures/reference-save.json"));
        fs::write(&path, encoded).unwrap();
        path
    }

    fn command(&self, arguments: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_gyrognome"))
            .args(arguments)
            .env("XDG_DATA_HOME", &self.0)
            .output()
            .unwrap()
    }

    fn command_with_input(&self, arguments: &[&str], input: &str) -> Output {
        self.command_with_input_and_env(arguments, input, &[])
    }

    fn command_with_input_and_env(
        &self,
        arguments: &[&str],
        input: &str,
        environment: &[(&str, &str)],
    ) -> Output {
        let mut child = Command::new(env!("CARGO_BIN_EXE_gyrognome"))
            .args(arguments)
            .env("XDG_DATA_HOME", &self.0)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .envs(environment.iter().copied())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(input.as_bytes())
            .unwrap();
        child.wait_with_output().unwrap()
    }

    #[cfg(feature = "enrollment-test-transport")]
    fn command_with_env(&self, arguments: &[&str], environment: &[(&str, &str)]) -> Output {
        let mut command = Command::new(env!("CARGO_BIN_EXE_gyrognome"));
        command.args(arguments).env("XDG_DATA_HOME", &self.0);
        for (key, value) in environment {
            command.env(key, value);
        }
        command.output().unwrap()
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).unwrap()
}

fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).unwrap()
}

#[test]
fn cli_registers_and_inspects_only_credential_safe_state() {
    let directory = TestDirectory::new("cli");
    let save = directory.save_file();
    let registration = directory.command(&["register", save.to_str().unwrap()]);
    assert!(registration.status.success(), "{}", stderr(&registration));
    let registration_text = stdout(&registration);
    let id = registration_text
        .strip_prefix("Registered managed character: ")
        .unwrap()
        .split_whitespace()
        .next()
        .unwrap();
    CharacterId::parse(id).unwrap();
    assert!(!registration_text.contains("4242"));
    assert!(!registration_text.contains("unrecognized-future-field"));

    let inspection = directory.command(&["managed-inspect", id, "--json"]);
    assert!(inspection.status.success(), "{}", stderr(&inspection));
    let inspection_text = stdout(&inspection);
    assert!(inspection_text.contains("Reference Hero"));
    assert!(!inspection_text.contains("4242"));
    assert!(!inspection_text.contains("unrecognized-future-field"));

    let listed = directory.command(&["list", "--json"]);
    assert!(listed.status.success(), "{}", stderr(&listed));
    let listed_text = stdout(&listed);
    assert!(listed_text.contains(id));
    assert!(!listed_text.contains("4242"));
    assert!(!listed_text.contains("unrecognized-future-field"));
}

#[test]
fn cli_reports_missing_managed_identifiers_without_sensitive_data() {
    let directory = TestDirectory::new("missing");
    let missing = directory.command(&["managed-inspect", "00000000-0000-4000-8000-000000000000"]);
    assert!(!missing.status.success());
    let diagnostic = stderr(&missing);
    assert!(diagnostic.contains("was not found"));
    assert!(!diagnostic.contains("4242"));
    assert!(!diagnostic.contains("unrecognized-future-field"));
}

#[test]
fn cli_creates_and_registers_an_offline_character_from_complete_traits() {
    let directory = TestDirectory::new("new-guy");
    let output = directory.command(&[
        "new-guy",
        "--name",
        "Offline Hero",
        "--race",
        "Gyrognome",
        "--class",
        "Robot Monk",
        "--json",
    ]);
    assert!(output.status.success(), "{}", stderr(&output));
    let registration: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let id = registration["id"].as_str().unwrap();
    assert_eq!(registration["identity"]["name"], "Offline Hero");
    assert_eq!(registration["identity"]["race"], "Gyrognome");
    assert_eq!(registration["identity"]["class"], "Robot Monk");
    assert!(registration["state"]["online"].is_null());
    assert!(!stdout(&output).contains("passkey"));

    let persisted = directory.command(&["managed-inspect", id, "--json"]);
    assert!(persisted.status.success(), "{}", stderr(&persisted));
    assert!(stdout(&persisted).contains("Offline Hero"));
    assert!(stdout(&persisted).contains("\"online\": null"));
}

#[cfg(feature = "enrollment-test-transport")]
#[test]
fn cli_enrolls_online_characters_and_redacts_persisted_output() {
    let directory = TestDirectory::new("online-new-guy");
    let output = directory.command_with_env(
        &["new-guy", "--json"],
        &[("GYROGNOME_TEST_WIZARD", "online")],
    );
    assert!(output.status.success(), "{}", stderr(&output));
    let registration: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let id = registration["id"].as_str().unwrap();
    assert_eq!(registration["state"]["online"]["realm"], "Alpaquil");
    assert!(!stdout(&output).contains("passkey"));

    let persisted = directory.command(&["managed-inspect", id, "--json"]);
    assert!(persisted.status.success(), "{}", stderr(&persisted));
    assert!(stdout(&persisted).contains("\"realm\": \"Alpaquil\""));
    assert!(!stdout(&persisted).contains("passkey"));
}

#[cfg(feature = "enrollment-test-transport")]
#[test]
fn cli_handles_duplicate_cancellation_and_invalid_online_drafts_without_registration() {
    let directory = TestDirectory::new("online-new-guy-no-registration");
    let duplicate = directory.command_with_env(
        &["new-guy", "--json"],
        &[
            ("GYROGNOME_TEST_WIZARD", "duplicate-correct"),
            ("GYROGNOME_TEST_CREATE", "duplicate,success"),
        ],
    );
    assert!(duplicate.status.success(), "{}", stderr(&duplicate));
    assert!(stdout(&duplicate).contains("\"online\""));

    let cancellation =
        directory.command_with_env(&["new-guy"], &[("GYROGNOME_TEST_WIZARD", "cancel")]);
    assert!(cancellation.status.success(), "{}", stderr(&cancellation));

    let invalid = directory.command_with_env(&["new-guy"], &[("GYROGNOME_TEST_WIZARD", "invalid")]);
    assert!(invalid.status.success(), "{}", stderr(&invalid));
}

#[cfg(feature = "enrollment-test-transport")]
#[test]
fn cli_stops_evidence_and_incomplete_online_enrollment_safely() {
    for environment in [
        vec![
            ("GYROGNOME_TEST_WIZARD", "online"),
            ("GYROGNOME_TEST_EVIDENCE", "malformed"),
        ],
        vec![
            ("GYROGNOME_TEST_WIZARD", "online"),
            ("GYROGNOME_TEST_CREATE", "incomplete"),
        ],
        vec![
            ("GYROGNOME_TEST_WIZARD", "online"),
            ("GYROGNOME_TEST_REPORT", "failed"),
        ],
    ] {
        let directory = TestDirectory::new("online-new-guy-failure");
        let output = directory.command_with_env(&["new-guy"], &environment);
        assert!(!output.status.success());
        let diagnostic = stderr(&output);
        assert!(!diagnostic.contains("passkey"));
        assert!(!diagnostic.contains("cmd="));
        let listed = directory.command(&["list", "--json"]);
        assert_eq!(stdout(&listed).trim(), "[]");
    }
}

#[test]
fn cli_rejects_partial_or_invalid_offline_creation_without_registration() {
    let directory = TestDirectory::new("new-guy-invalid");
    let partial = directory.command(&["new-guy", "--name", "Offline Hero"]);
    assert!(!partial.status.success());
    assert!(stderr(&partial).contains("requires --name, --race, and --class together"));

    let invalid = directory.command(&[
        "new-guy",
        "--name",
        "Offline Hero",
        "--race",
        "Invalid Race",
        "--class",
        "Robot Monk",
    ]);
    assert!(!invalid.status.success());
    assert!(stderr(&invalid).contains("race is not present"));

    for name in ["", "   ", "x\u{7f}", "x".repeat(31).as_str()] {
        let invalid_name = directory.command(&[
            "new-guy",
            "--name",
            name,
            "--race",
            "Gyrognome",
            "--class",
            "Robot Monk",
        ]);
        assert!(
            !invalid_name.status.success(),
            "{name:?} should be rejected"
        );
    }

    let listed = directory.command(&["list", "--json"]);
    assert!(listed.status.success(), "{}", stderr(&listed));
    assert_eq!(stdout(&listed).trim(), "[]");
}

#[test]
fn dashboard_rejects_missing_identifiers_before_entering_terminal_mode() {
    let directory = TestDirectory::new("dashboard-missing");
    let missing = directory.command(&["dashboard", "00000000-0000-4000-8000-000000000000"]);
    assert!(!missing.status.success());
    assert!(stderr(&missing).contains("was not found"));
    let output = [stdout(&missing), stderr(&missing)].join("");
    assert!(!output.contains("\u{1b}[?1049h"));
    assert!(!output.contains("4242"));
}

#[test]
fn dashboard_without_registrations_reports_an_error_before_terminal_mode() {
    let directory = TestDirectory::new("dashboard-empty");
    let output = directory.command(&["dashboard"]);
    assert!(!output.status.success());
    assert!(stderr(&output).contains("no managed characters are registered"));
    assert!(
        ![stdout(&output), stderr(&output)]
            .join("")
            .contains("\u{1b}[?1049h")
    );
}

#[test]
fn dashboard_selection_entries_are_newest_first_with_credential_safe_activity() {
    let directory = TestDirectory::new("dashboard-selection");
    let character =
        save::import_text(&STANDARD.encode(include_str!("fixtures/reference-save.json"))).unwrap();
    let mut store = Store::open_at(directory.0.join("gyrognome")).unwrap();
    let older = store.register(&character).unwrap();
    thread::sleep(Duration::from_millis(5));
    let newer = store.register(&character).unwrap();

    let entries = dashboard::selector_entries(&store, |id| {
        if id == &older.id {
            dashboard::SelectorActivity::Unavailable
        } else {
            dashboard::SelectorActivity::Active
        }
    })
    .unwrap();

    assert_eq!(
        entries
            .iter()
            .map(|entry| entry.id.to_string())
            .collect::<Vec<_>>(),
        [newer.id.to_string(), older.id.to_string()]
    );
    assert_eq!(entries[0].activity, dashboard::SelectorActivity::Active);
    assert_eq!(
        entries[1].activity,
        dashboard::SelectorActivity::Unavailable
    );
    assert_eq!(entries[0].last_accessed_unix_ms, newer.updated_at_unix_ms);
    assert_eq!(entries[0].identity.name, "Reference Hero");
}

#[test]
fn cli_deletes_only_after_explicit_confirmation_without_sensitive_output() {
    let directory = TestDirectory::new("delete");
    let save = directory.save_file();
    let registration = directory.command(&["register", save.to_str().unwrap()]);
    let id = stdout(&registration)
        .strip_prefix("Registered managed character: ")
        .unwrap()
        .split_whitespace()
        .next()
        .unwrap()
        .to_owned();

    let cancelled = directory.command_with_input(&["delete", &id], "no\n");
    assert!(cancelled.status.success(), "{}", stderr(&cancelled));
    assert!(stdout(&cancelled).contains("Deletion cancelled."));
    assert!(
        directory
            .command(&["managed-inspect", &id])
            .status
            .success()
    );

    let deleted = directory.command_with_input(&["delete", &id], "yes\n");
    assert!(deleted.status.success(), "{}", stderr(&deleted));
    let output = [stdout(&deleted), stderr(&deleted)].join("");
    assert!(output.contains("Deleted managed character"));
    assert!(!output.contains("4242"));
    assert!(!output.contains("unrecognized-future-field"));
    assert!(
        !directory
            .command(&["managed-inspect", &id])
            .status
            .success()
    );
}

#[test]
fn cli_declined_report_sends_no_request_and_keeps_safe_output() {
    let directory = TestDirectory::new("report-cancelled");
    let save = directory.save_file();
    let registration = directory.command(&["register", save.to_str().unwrap()]);
    let id = stdout(&registration)
        .strip_prefix("Registered managed character: ")
        .unwrap()
        .split_whitespace()
        .next()
        .unwrap()
        .to_owned();

    let cancelled = directory.command_with_input(&["report", &id], "no\n");
    assert!(cancelled.status.success(), "{}", stderr(&cancelled));
    let output = [stdout(&cancelled), stderr(&cancelled)].join("");
    assert!(output.contains("Report cancelled."));
    assert!(!output.contains("4242"));
    assert!(!output.contains("unrecognized-future-field"));
    assert!(
        directory
            .command(&["managed-inspect", &id])
            .status
            .success()
    );
}

#[cfg(feature = "enrollment-test-transport")]
#[test]
fn cli_confirmed_report_succeeds_while_character_is_running() {
    let directory = TestDirectory::new("report-owned");
    let mut character =
        save::import_text(&STANDARD.encode(include_str!("fixtures/reference-save.json"))).unwrap();
    character.online.as_mut().unwrap().host = "https://progressquest.com/alpaquil.php?".to_owned();
    let mut store = Store::open_at(directory.0.join("gyrognome")).unwrap();
    let registered = store.register(&character).unwrap();
    let before = serde_json::to_value(store.get(&registered.id).unwrap().state).unwrap();
    let worker = Worker::start(
        Store::open_at(directory.0.join("gyrognome")).unwrap(),
        registered.id.clone(),
    )
    .unwrap();
    let action_log = directory.0.join("actions");
    let id = registered.id.to_string();

    let reported = directory.command_with_input_and_env(
        &["report", &id],
        "yes\n",
        &[
            ("GYROGNOME_TEST_ACTION_LOG", action_log.to_str().unwrap()),
            ("GYROGNOME_TEST_REPORT", "delivered"),
        ],
    );

    assert!(reported.status.success(), "{}", stderr(&reported));
    let output = [stdout(&reported), stderr(&reported)].join("");
    assert!(output.contains("Leaderboard report delivered."));
    assert!(!output.contains("already running"));
    assert!(!output.contains("4242"));
    assert!(!output.contains("unrecognized-future-field"));
    assert!(!output.contains("passkey"));
    assert_eq!(fs::read_to_string(action_log).unwrap(), "report\n");
    assert!(store.is_owned(&registered.id).unwrap());
    assert_eq!(
        serde_json::to_value(store.get(&registered.id).unwrap().state).unwrap(),
        before
    );
    drop(worker);
}

#[test]
fn cli_delete_reports_invalid_and_unknown_identifiers() {
    let directory = TestDirectory::new("delete-errors");
    let invalid = directory.command_with_input(&["delete", "not-an-id"], "yes\n");
    assert!(!invalid.status.success());
    assert!(stderr(&invalid).contains("identifier is invalid"));

    let missing =
        directory.command_with_input(&["delete", "00000000-0000-4000-8000-000000000000"], "yes\n");
    assert!(!missing.status.success());
    assert!(stderr(&missing).contains("was not found"));
}

#[test]
fn cli_refuses_to_delete_a_character_owned_by_a_worker() {
    let directory = TestDirectory::new("delete-owned");
    let character =
        save::import_text(&STANDARD.encode(include_str!("fixtures/reference-save.json"))).unwrap();
    let mut store = Store::open_at(directory.0.join("gyrognome")).unwrap();
    let registered = store.register(&character).unwrap();
    let worker = Worker::start(
        Store::open_at(directory.0.join("gyrognome")).unwrap(),
        registered.id.clone(),
    )
    .unwrap();

    let deleted = directory.command_with_input(&["delete", &registered.id.to_string()], "yes\n");
    assert!(!deleted.status.success());
    assert!(stderr(&deleted).contains("is running; stop it before deleting"));
    assert!(store.get(&registered.id).is_ok());
    drop(worker);
}

#[test]
fn stopping_a_worker_keeps_its_final_persisted_state_readable() {
    let directory = TestDirectory::new("stop");
    let character =
        save::import_text(&STANDARD.encode(include_str!("fixtures/reference-save.json"))).unwrap();
    let mut store = Store::open_at(&directory.0).unwrap();
    let registered = store.register(&character).unwrap();
    drop(store);

    let stop = Arc::new(AtomicBool::new(false));
    let worker =
        Worker::start(Store::open_at(&directory.0).unwrap(), registered.id.clone()).unwrap();
    let worker_stop = Arc::clone(&stop);
    let join = thread::spawn(move || worker.run_until(&worker_stop, Duration::from_millis(2)));
    thread::sleep(Duration::from_millis(20));
    stop.store(true, Ordering::Relaxed);
    join.join().unwrap().unwrap();

    let final_state = Store::open_at(&directory.0)
        .unwrap()
        .get(&registered.id)
        .unwrap();
    assert!(final_state.state.progress.task.position > registered.state.progress.task.position);
    assert_eq!(final_state.state.document, serde_json::Value::Null);
}
