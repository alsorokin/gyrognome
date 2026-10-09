use std::{
    fs,
    io::Write,
    os::unix::fs::PermissionsExt,
    path::PathBuf,
    process::{Command, Output, Stdio},
};

use gyrognome::{
    newguy::{self, Selection},
    runtime::{CharacterId, Store, Worker},
};
use uuid::Uuid;

struct TestDirectory(PathBuf);

impl TestDirectory {
    fn new() -> Self {
        let path = std::env::current_dir()
            .unwrap()
            .join("target/gyrognome-autostart-tests")
            .join(Uuid::new_v4().to_string());
        fs::create_dir_all(path.join("config/systemd/user")).unwrap();
        fs::create_dir_all(path.join("runtime")).unwrap();
        Self(path)
    }

    fn register(&self) -> CharacterId {
        let character = newguy::generate_local(
            &Selection {
                name: "Autostart Test".to_owned(),
                race: "Half Orc".to_owned(),
                class: "Ur-Paladin".to_owned(),
            },
            &gyrognome::ruleset::BUNDLED,
        )
        .unwrap();
        Store::open_at(self.0.join("data/gyrognome"))
            .unwrap()
            .register(&character)
            .unwrap()
            .id
    }

    fn command(&self, args: &[&str]) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_gyro"));
        command
            .args(args)
            .env("HOME", &self.0)
            .env("XDG_CONFIG_HOME", self.0.join("config"))
            .env("XDG_DATA_HOME", self.0.join("data"))
            .env("XDG_RUNTIME_DIR", self.0.join("runtime"))
            .env("SYSTEMD_UNIT_PATH", self.0.join("config/systemd/user"))
            .env(
                "DBUS_SESSION_BUS_ADDRESS",
                format!("unix:path={}/missing-bus", self.0.display()),
            );
        if self.0.join("bin/systemctl").exists() {
            command.env("PATH", self.0.join("bin"));
        }
        command
    }

    fn run(&self, args: &[&str]) -> Output {
        self.command(args).output().unwrap()
    }

    fn delete(&self, id: &CharacterId, confirmation: &str) -> Output {
        let mut child = self
            .command(&["delete", &id.to_string()])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(confirmation.as_bytes())
            .unwrap();
        child.wait_with_output().unwrap()
    }

    fn fake_systemctl(&self) {
        fs::create_dir_all(self.0.join("bin")).unwrap();
        let script = self.0.join("bin/systemctl");
        fs::write(
            &script,
            r#"#!/bin/sh
printf '%s\n' "$*" >> "$HOME/actions"
case "$*" in
  *is-active*)
    if [ -f "$HOME/active" ]; then printf 'active\n'; exit 0; fi
    printf 'inactive\n'; exit 3 ;;
  *is-enabled*)
    if [ -f "$HOME/unsupported" ]; then printf 'masked\n'; exit 1; fi
    if [ -f "$HOME/enabled" ]; then printf 'enabled\n'; exit 0; fi
    printf 'disabled\n'; exit 1 ;;
  *disable*)
    if [ -f "$HOME/fail" ]; then printf 'permission denied\n' >&2; exit 1; fi
    /usr/bin/rm -f "$HOME/enabled"; exit 0 ;;
  *enable*)
    if [ -f "$HOME/fail" ]; then printf 'permission denied\n' >&2; exit 1; fi
    : > "$HOME/enabled"; exit 0 ;;
  *) printf 'unexpected action\n' >&2; exit 1 ;;
esac
"#,
        )
        .unwrap();
        fs::set_permissions(script, fs::Permissions::from_mode(0o700)).unwrap();
    }

    fn assert_registered(&self, id: &CharacterId) {
        assert!(
            Store::open_at(self.0.join("data/gyrognome"))
                .unwrap()
                .identity(id)
                .is_ok()
        );
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

fn text(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

#[test]
fn cli_settings_and_text_json_status_are_independent_of_activity() {
    let directory = TestDirectory::new();
    directory.fake_systemctl();
    let id = directory.register().to_string();
    for (setting, expected) in [
        ("on", "enabled"),
        ("on", "enabled"),
        ("off", "disabled"),
        ("off", "disabled"),
    ] {
        let output = directory.run(&["autostart", &id, setting]);
        assert!(output.status.success(), "{}", text(&output));
        assert!(text(&output).contains("Current worker activity is unchanged"));
        let status = directory.run(&["status", &id, "--json"]);
        assert!(status.status.success(), "{}", text(&status));
        let status: serde_json::Value = serde_json::from_slice(&status.stdout).unwrap();
        assert_eq!(status["autostart"], expected);
        assert_eq!(status["service"], "inactive");
        assert_eq!(status["runtime_owned"], false);
    }
    let output = directory.run(&["status", &id]);
    assert!(text(&output).contains("Autostart: Off"));
    fs::write(directory.0.join("active"), "").unwrap();
    let output = directory.run(&["autostart", &id, "on"]);
    assert!(output.status.success(), "{}", text(&output));
    let status = directory.run(&["status", &id, "--json"]);
    let status: serde_json::Value = serde_json::from_slice(&status.stdout).unwrap();
    assert_eq!(status["service"], "active");
    assert_eq!(status["autostart"], "enabled");
    fs::write(directory.0.join("unsupported"), "").unwrap();
    let output = directory.run(&["status", &id, "--json"]);
    let status: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(status["service"], "active");
    assert_eq!(status["autostart"], "unavailable");
    assert!(
        status["autostart_diagnostic"]
            .as_str()
            .unwrap()
            .contains("masked")
    );
    let actions = fs::read_to_string(directory.0.join("actions")).unwrap();
    assert!(!actions.lines().any(|line| line.contains(" start ")
        || line.contains(" stop ")
        || line.contains("--now")
        || line.contains("--runtime")));
}

#[test]
fn cli_errors_validate_identifiers_before_touching_services() {
    let directory = TestDirectory::new();
    directory.fake_systemctl();
    let id = directory.register();
    for id in ["invalid".to_owned(), CharacterId::new().to_string()] {
        let output = directory.run(&["autostart", &id, "on"]);
        assert!(!output.status.success());
    }
    assert!(!directory.0.join("actions").exists());
    fs::write(directory.0.join("fail"), "").unwrap();
    let output = directory.run(&["autostart", &id.to_string(), "on"]);
    assert!(!output.status.success());
    assert!(text(&output).contains("permission denied"));
    assert!(!String::from_utf8_lossy(&output.stdout).contains("Autostart On"));
    directory.assert_registered(&id);
}

#[test]
fn cli_deletion_cancellation_ownership_and_cleanup_failures_preserve_preferences() {
    let directory = TestDirectory::new();
    directory.fake_systemctl();
    let id = directory.register();
    fs::write(directory.0.join("enabled"), "").unwrap();
    assert!(directory.delete(&id, "no\n").status.success());
    assert!(!directory.0.join("actions").exists());
    let worker = Worker::start(
        Store::open_at(directory.0.join("data/gyrognome")).unwrap(),
        id.clone(),
    )
    .unwrap();
    let output = directory.delete(&id, "yes\n");
    assert!(!output.status.success());
    assert!(!directory.0.join("actions").exists());
    drop(worker);
    fs::write(directory.0.join("active"), "").unwrap();
    let output = directory.delete(&id, "yes\n");
    assert!(!output.status.success());
    assert!(directory.0.join("enabled").exists());
    fs::remove_file(directory.0.join("active")).unwrap();
    fs::write(directory.0.join("fail"), "").unwrap();
    let output = directory.delete(&id, "yes\n");
    assert!(!output.status.success());
    assert!(text(&output).contains("permission denied"));
    directory.assert_registered(&id);
    assert!(directory.0.join("enabled").exists());
    fs::remove_file(directory.0.join("fail")).unwrap();
    let output = directory.delete(&id, "yes\n");
    assert!(output.status.success(), "{}", text(&output));
    assert!(!directory.0.join("enabled").exists());
    assert!(directory.run(&["list", "--json"]).status.success());
}

#[test]
fn real_systemd_persistent_links_work_without_a_user_bus() {
    let directory = TestDirectory::new();
    let id = directory.register();
    fs::write(
        directory.0.join("config/systemd/user/gyrognome@.service"),
        include_str!("../systemd/user/gyrognome@.service"),
    )
    .unwrap();
    let unit = format!("gyrognome@{id}.service");
    let link = directory
        .0
        .join("config/systemd/user/default.target.wants")
        .join(&unit);
    for _ in 0..2 {
        let output = directory.run(&["autostart", &id.to_string(), "on"]);
        assert!(output.status.success(), "{}", text(&output));
        assert!(link.is_symlink());
        assert!(
            fs::read_to_string(&link)
                .unwrap()
                .contains("WantedBy=default.target")
        );
        let store = Store::open_at(directory.0.join("data/gyrognome")).unwrap();
        assert!(!store.is_owned(&id).unwrap());
    }
    let worker = Worker::start(
        Store::open_at(directory.0.join("data/gyrognome")).unwrap(),
        id.clone(),
    )
    .unwrap();
    let output = directory.run(&["autostart", &id.to_string(), "off"]);
    assert!(output.status.success(), "{}", text(&output));
    assert!(!link.is_symlink());
    assert!(
        Store::open_at(directory.0.join("data/gyrognome"))
            .unwrap()
            .is_owned(&id)
            .unwrap()
    );
    let output = directory.run(&["autostart", &id.to_string(), "on"]);
    assert!(output.status.success(), "{}", text(&output));
    drop(worker);
    let output = directory.delete(&id, "yes\n");
    assert!(output.status.success(), "{}", text(&output));
    assert!(!link.is_symlink());
}

#[test]
fn real_deletion_handles_absent_and_removed_templates() {
    let directory = TestDirectory::new();
    let id = directory.register();
    let output = directory.delete(&id, "yes\n");
    assert!(output.status.success(), "{}", text(&output));

    let id = directory.register();
    let template = directory.0.join("config/systemd/user/gyrognome@.service");
    fs::write(
        &template,
        include_str!("../systemd/user/gyrognome@.service"),
    )
    .unwrap();
    let output = directory.run(&["autostart", &id.to_string(), "on"]);
    assert!(output.status.success(), "{}", text(&output));
    let link = directory
        .0
        .join("config/systemd/user/default.target.wants")
        .join(format!("gyrognome@{id}.service"));
    fs::remove_file(template).unwrap();
    let output = directory.delete(&id, "yes\n");
    assert!(output.status.success(), "{}", text(&output));
    assert!(
        !link.is_symlink(),
        "deletion left a dangling startup registration"
    );
}
