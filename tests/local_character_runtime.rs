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
    time::{Duration, Instant},
};

use base64::{Engine, engine::general_purpose::STANDARD};
use gyrognome::{
    dashboard,
    desktop_save::{
        DesktopAdaptations, DesktopQuestMarker, DesktopValidatedBar, DesktopValidatedBars,
        DesktopValidatedPrivateMetadata, DesktopValidatedProfile, DesktopValidatedRow,
        DesktopValidatedSave,
    },
    newguy::RandomSource,
    runtime::{CharacterId, ManagedInspectionState, Store, Worker},
    save,
};
use rusqlite::Connection;
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
        Command::new(env!("CARGO_BIN_EXE_gyro"))
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
        let mut child = Command::new(env!("CARGO_BIN_EXE_gyro"))
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

    fn command_in_pty(&self, arguments: &[&str], input: &[&str]) -> Output {
        self.command_in_pty_with_initial_delay(arguments, input, Duration::from_millis(100))
    }

    fn command_in_pty_with_initial_delay(
        &self,
        arguments: &[&str],
        input: &[&str],
        initial_delay: Duration,
    ) -> Output {
        let dashboard_command = std::iter::once(env!("CARGO_BIN_EXE_gyro"))
            .chain(arguments.iter().copied())
            .map(shell_quote)
            .collect::<Vec<_>>()
            .join(" ");
        let transcript = self.0.join("pty.typescript");
        let before = self.0.join("pty.termios.before");
        let after = self.0.join("pty.termios.after");
        let command = format!(
            "stty -g > {}; {}; status=$?; stty -g > {}; exit \"$status\"",
            shell_quote(before.to_str().unwrap()),
            dashboard_command,
            shell_quote(after.to_str().unwrap())
        );
        let mut child = Command::new("script")
            .args([
                "--quiet",
                "--return",
                "--command",
                &command,
                transcript.to_str().unwrap(),
            ])
            .env("XDG_DATA_HOME", &self.0)
            .env("TERM", "xterm-256color")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let mut stdin = child.stdin.take().unwrap();
        for (index, chunk) in input.iter().enumerate() {
            thread::sleep(if index == 0 {
                initial_delay
            } else {
                Duration::from_millis(100)
            });
            if let Err(error) = stdin.write_all(chunk.as_bytes()) {
                if error.kind() == std::io::ErrorKind::BrokenPipe {
                    break;
                }
                panic!("failed to send terminal input: {error}");
            }
        }
        drop(stdin);
        let mut output = child.wait_with_output().unwrap();
        output.stdout = fs::read(transcript).unwrap();
        output
    }

    #[cfg(feature = "enrollment-test-transport")]
    fn command_with_env(&self, arguments: &[&str], environment: &[(&str, &str)]) -> Output {
        let mut command = Command::new(env!("CARGO_BIN_EXE_gyro"));
        command.args(arguments).env("XDG_DATA_HOME", &self.0);
        for (key, value) in environment {
            command.env(key, value);
        }
        command.output().unwrap()
    }
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
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
    let inspected: serde_json::Value = serde_json::from_slice(&inspection.stdout).unwrap();
    assert!(inspected["rested"]["availableMs"].as_u64().is_some());
    assert_eq!(inspected["rested"]["activeMultiplier"], 1);
    let human = directory.command(&["managed-inspect", id]);
    assert!(human.status.success(), "{}", stderr(&human));
    assert!(stdout(&human).contains("Rested:"));
    assert!(stdout(&human).contains("Rested-timeline leaderboard acceptance"));

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
    for args in [
        &["dashboard"][..],
        &["dashboard", "--resume"][..],
        &["dashboard", "-r"][..],
    ] {
        let output = directory.command(args);
        assert!(!output.status.success());
        assert!(stderr(&output).contains("no managed characters are registered"));
        assert!(
            ![stdout(&output), stderr(&output)]
                .join("")
                .contains("\u{1b}[?1049h")
        );
    }
}

#[test]
fn dashboard_resume_propagates_storage_errors_instead_of_falling_back() {
    let directory = TestDirectory::new("dashboard-resume-storage-error");
    let store = Store::open_at(directory.0.join("gyrognome")).unwrap();
    drop(store);
    let connection = Connection::open(directory.0.join("gyrognome/characters.sqlite3")).unwrap();
    connection.execute_batch("DROP TABLE characters;").unwrap();
    drop(connection);

    let output = directory.command(&["dashboard", "--resume"]);
    assert!(!output.status.success());
    assert!(stderr(&output).contains("no such table: characters"));
    assert!(!stdout(&output).contains("Select a managed character"));
}

#[test]
fn dashboard_failed_terminal_startup_does_not_record_an_access() {
    let directory = TestDirectory::new("dashboard-startup-failure");
    let character =
        save::import_text(&STANDARD.encode(include_str!("fixtures/reference-save.json"))).unwrap();
    let mut store = Store::open_at(directory.0.join("gyrognome")).unwrap();
    let registered = store.register(&character).unwrap();
    drop(store);

    let output = directory.command(&["dashboard", &registered.id.to_string()]);
    assert!(!output.status.success());
    let store = Store::open_at(directory.0.join("gyrognome")).unwrap();
    assert_eq!(
        store.get(&registered.id).unwrap().updated_at_unix_ms,
        0,
        "failed terminal setup must not record an opening"
    );
}

#[test]
fn dashboard_refreshes_do_not_repeat_the_opening_write() {
    let directory = TestDirectory::new("dashboard-single-access-write");
    let character =
        save::import_text(&STANDARD.encode(include_str!("fixtures/reference-save.json"))).unwrap();
    let mut store = Store::open_at(directory.0.join("gyrognome")).unwrap();
    let registered = store.register(&character).unwrap();
    drop(store);
    let connection = Connection::open(directory.0.join("gyrognome/characters.sqlite3")).unwrap();
    connection
        .execute_batch(
            "CREATE TABLE dashboard_access_writes (count INTEGER NOT NULL);
             INSERT INTO dashboard_access_writes VALUES (0);
             CREATE TRIGGER count_dashboard_access
             AFTER UPDATE OF updated_at_unix_ms ON characters
             BEGIN
                 UPDATE dashboard_access_writes SET count = count + 1;
             END;",
        )
        .unwrap();
    drop(connection);

    let output = directory.command_in_pty_with_initial_delay(
        &[
            "dashboard",
            &registered.id.to_string(),
            "--refresh-ms",
            "100",
        ],
        &["q"],
        Duration::from_millis(600),
    );
    assert!(output.status.success(), "{}", stdout(&output));

    let connection = Connection::open(directory.0.join("gyrognome/characters.sqlite3")).unwrap();
    let count: i64 = connection
        .query_row("SELECT count FROM dashboard_access_writes", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(count, 1);
}

#[test]
fn dashboard_selection_entries_are_newest_first_with_credential_safe_activity() {
    let directory = TestDirectory::new("dashboard-selection");
    let character =
        save::import_text(&STANDARD.encode(include_str!("fixtures/reference-save.json"))).unwrap();
    let mut store = Store::open_at(directory.0.join("gyrognome")).unwrap();
    let older = store.register(&character).unwrap();
    let worker_state_loaded_before_open = store.get(&older.id).unwrap().state;
    store.record_dashboard_access(&older.id).unwrap();
    thread::sleep(Duration::from_millis(5));
    let newer = store.register(&character).unwrap();
    store.record_dashboard_access(&newer.id).unwrap();
    store
        .replace_profile(
            &older.id,
            &gyrognome::state::OnlineProfile {
                motto: "Does not change recency".to_owned(),
                guild: String::new(),
            },
        )
        .unwrap();
    store
        .replace_state(&older.id, &worker_state_loaded_before_open)
        .unwrap();

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
    assert_ne!(entries[0].last_accessed_unix_ms, 0);
    assert_eq!(
        dashboard::most_recently_accessed_character(&store).unwrap(),
        Some(newer.id.clone())
    );
    assert_eq!(entries[0].identity.name, "Reference Hero");
    let older_entry = &entries[1];
    assert!(older_entry.rested.available_ms >= 5);
    assert_eq!(older_entry.rested.active_multiplier, 1);
    assert_eq!(entries[0].rested.active_multiplier, 1);
    assert_eq!(
        store.get(&older.id).unwrap().updated_at_unix_ms,
        entries[1].last_accessed_unix_ms
    );
}

#[test]
fn dashboard_resume_end_to_end_uses_access_order_and_selector_fallback() {
    let directory = TestDirectory::new("dashboard-resume-pty");
    let character =
        save::import_text(&STANDARD.encode(include_str!("fixtures/reference-save.json"))).unwrap();
    let mut store = Store::open_at(directory.0.join("gyrognome")).unwrap();
    let browser = store.register(&character).unwrap();
    let desktop = store
        .register_desktop(&synthetic_desktop_save(), &mut SyntheticRandom(17))
        .unwrap();
    drop(store);

    let opened_browser = directory.command_in_pty(&["dashboard", &browser.id.to_string()], &["q"]);
    assert!(
        opened_browser.status.success(),
        "{}",
        stderr(&opened_browser)
    );
    thread::sleep(Duration::from_millis(5));

    let opened_desktop = directory.command_in_pty(&["dashboard"], &["j", "\r", "q"]);
    assert!(
        opened_desktop.status.success(),
        "{}",
        stderr(&opened_desktop)
    );

    let mut store = Store::open_at(directory.0.join("gyrognome")).unwrap();
    let never_opened = store.register(&character).unwrap();
    let browser_access = store.get(&browser.id).unwrap().updated_at_unix_ms;
    let desktop_access = store.get_desktop(&desktop.id).unwrap().updated_at_unix_ms;
    assert!(
        desktop_access > browser_access,
        "desktop={desktop_access}, browser={browser_access}; selector output: {}",
        stdout(&opened_desktop)
    );
    let stale_browser_state = store.get(&browser.id).unwrap().state;
    store
        .replace_state(&browser.id, &stale_browser_state)
        .unwrap();
    let entries =
        dashboard::selector_entries(&store, |_| dashboard::SelectorActivity::Inactive).unwrap();
    assert_eq!(entries[0].id, desktop.id);
    assert_eq!(
        dashboard::most_recently_accessed_character(&store).unwrap(),
        Some(desktop.id.clone())
    );
    drop(store);

    for flag in ["--resume", "-r"] {
        let previous_desktop_access = Store::open_at(directory.0.join("gyrognome"))
            .unwrap()
            .get_desktop(&desktop.id)
            .unwrap()
            .updated_at_unix_ms;
        thread::sleep(Duration::from_millis(5));
        let resumed = directory.command_in_pty(&["dashboard", flag], &["q"]);
        assert!(resumed.status.success(), "{}", stderr(&resumed));
        let store = Store::open_at(directory.0.join("gyrognome")).unwrap();
        assert!(
            store.get_desktop(&desktop.id).unwrap().updated_at_unix_ms > previous_desktop_access
        );
        assert_eq!(
            store.get(&browser.id).unwrap().updated_at_unix_ms,
            browser_access
        );
    }

    let mut store = Store::open_at(directory.0.join("gyrognome")).unwrap();
    store.remove(&desktop.id).unwrap();
    drop(store);
    let previous_browser_access = Store::open_at(directory.0.join("gyrognome"))
        .unwrap()
        .get(&browser.id)
        .unwrap()
        .updated_at_unix_ms;
    thread::sleep(Duration::from_millis(5));
    let resumed_browser = directory.command_in_pty(&["dashboard", "-r"], &["q"]);
    assert!(
        resumed_browser.status.success(),
        "{}",
        stderr(&resumed_browser)
    );
    assert!(
        Store::open_at(directory.0.join("gyrognome"))
            .unwrap()
            .get(&browser.id)
            .unwrap()
            .updated_at_unix_ms
            > previous_browser_access
    );

    let mut store = Store::open_at(directory.0.join("gyrognome")).unwrap();
    store.remove(&browser.id).unwrap();
    drop(store);
    let fallback = directory.command_in_pty(&["dashboard", "--resume"], &["q"]);
    assert!(fallback.status.success(), "{}", stderr(&fallback));
    let store = Store::open_at(directory.0.join("gyrognome")).unwrap();
    assert_eq!(store.get(&never_opened.id).unwrap().updated_at_unix_ms, 0);
}

#[test]
fn dashboard_access_write_errors_propagate_after_terminal_cleanup() {
    let directory = TestDirectory::new("dashboard-access-error");
    let save = directory.save_file();
    let registered = directory.command(&["register", save.to_str().unwrap()]);
    assert!(registered.status.success(), "{}", stderr(&registered));
    let id = stdout(&registered)
        .strip_prefix("Registered managed character: ")
        .unwrap()
        .split_whitespace()
        .next()
        .unwrap()
        .to_owned();
    let connection = Connection::open(directory.0.join("gyrognome/characters.sqlite3")).unwrap();
    connection
        .execute_batch(
            "CREATE TRIGGER fail_dashboard_access
             BEFORE UPDATE OF updated_at_unix_ms ON characters
             BEGIN SELECT RAISE(ABORT, 'injected dashboard access failure'); END;",
        )
        .unwrap();
    drop(connection);

    let output = directory.command_in_pty(&["dashboard", &id], &["q"]);
    assert!(!output.status.success());
    assert!(
        stdout(&output).contains("injected dashboard access failure"),
        "{}",
        stdout(&output)
    );
    let before = fs::read(directory.0.join("pty.termios.before")).unwrap();
    let after = fs::read(directory.0.join("pty.termios.after")).unwrap();
    assert!(!before.is_empty(), "could not read the PTY terminal mode");
    assert_eq!(
        before, after,
        "terminal mode was not restored after the access write failed: {before:?} vs {after:?}"
    );
    let store = Store::open_at(directory.0.join("gyrognome")).unwrap();
    assert_eq!(
        store
            .get(&CharacterId::parse(&id).unwrap())
            .unwrap()
            .updated_at_unix_ms,
        0
    );
}

struct SyntheticRandom(u32);

impl RandomSource for SyntheticRandom {
    fn next_u32(&mut self) -> Result<u32, gyrognome::newguy::NewGuyError> {
        Ok(self.0)
    }
}

fn synthetic_desktop_save() -> DesktopValidatedSave {
    let rows = |values: &[(&str, &str)]| {
        values
            .iter()
            .map(|(caption, value)| DesktopValidatedRow {
                header: [0, -1, -1, 0, 1],
                caption: (*caption).to_owned(),
                subitems: vec![(*value).to_owned()],
            })
            .collect()
    };
    DesktopValidatedSave {
        traits: rows(&[
            ("Name", "Desktop Hero"),
            ("Race", "Gyrognome"),
            ("Class", "Robot Monk"),
            ("Level", "2"),
        ]),
        stats: rows(&[
            ("STR", "12"),
            ("CON", "11"),
            ("DEX", "10"),
            ("INT", "9"),
            ("WIS", "8"),
            ("CHA", "7"),
            ("HP Max", "20"),
            ("MP Max", "15"),
        ]),
        equipment: rows(&[
            ("Weapon", "Stick"),
            ("Shield", "Plate"),
            ("Helm", "Cap"),
            ("Hauberk", "Burlap"),
            ("Brassairts", "Cloth"),
            ("Vambraces", "Cloth"),
            ("Gauntlets", "Cloth"),
            ("Gambeson", "Cloth"),
            ("Cuisses", "Cloth"),
            ("Greaves", "Cloth"),
            ("Sollerets", "Cloth"),
        ]),
        inventory: rows(&[("Gold", "3")]),
        spells: Vec::new(),
        plots: Vec::new(),
        quests: Vec::new(),
        current_task: "load".to_owned(),
        quest: DesktopQuestMarker::None,
        queue: Vec::new(),
        activity: "Loading...".to_owned(),
        bars: DesktopValidatedBars {
            experience: DesktopValidatedBar {
                position: 0,
                maximum: 100,
            },
            encumbrance: DesktopValidatedBar {
                position: 0,
                maximum: 50,
            },
            plot: DesktopValidatedBar {
                position: 0,
                maximum: 26,
            },
            quest: DesktopValidatedBar {
                position: 0,
                maximum: 0,
            },
            task: DesktopValidatedBar {
                position: 0,
                maximum: 2_000,
            },
        },
        prized_equipment: 0,
        game_style: 3,
        profile: DesktopValidatedProfile {
            motto: "Synthetic motto".to_owned(),
            guild: "Synthetic Guild".to_owned(),
        },
        private: DesktopValidatedPrivateMetadata {
            passkey: 4_242,
            realm: "Synthetic Realm".to_owned(),
            endpoint: "https://synthetic.invalid/".to_owned(),
            account: "synthetic-account".to_owned(),
            password: "synthetic-password".to_owned(),
        },
        adaptations: DesktopAdaptations {
            legacy_prologue_62: false,
            legacy_quest_placeholder: false,
            spelling_patch_applied: false,
        },
        restored: None,
    }
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
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        let inspection = Store::open_at(&directory.0)
            .unwrap()
            .live_inspection(&registered.id)
            .unwrap();
        let ManagedInspectionState::Browser(live) = inspection.state else {
            panic!("expected browser state");
        };
        if live.state.progress.task.position > registered.state.progress.task.position {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "worker did not service a virtual tick"
        );
        thread::sleep(Duration::from_millis(5));
    }
    stop.store(true, Ordering::Relaxed);
    join.join().unwrap().unwrap();

    let final_state = Store::open_at(&directory.0)
        .unwrap()
        .get(&registered.id)
        .unwrap();
    assert!(final_state.state.progress.task.position > registered.state.progress.task.position);
    assert_eq!(final_state.state.document, serde_json::Value::Null);
}
