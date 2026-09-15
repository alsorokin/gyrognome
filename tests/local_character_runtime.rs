use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::Duration,
};

use base64::{Engine, engine::general_purpose::STANDARD};
use gyrognome::{
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
