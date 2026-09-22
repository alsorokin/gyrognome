#![cfg(feature = "enrollment-test-transport")]

use base64::{Engine, engine::general_purpose::STANDARD};
use gyrognome::{
    runtime::{CharacterId, Store, Worker},
    save,
};
use std::{
    fs,
    path::PathBuf,
    process::{Command, Output},
};

struct ProfileCli {
    root: PathBuf,
    id: CharacterId,
}

impl ProfileCli {
    fn new(online: bool) -> Self {
        let root = PathBuf::from("target/profile-cli-tests").join(uuid::Uuid::new_v4().to_string());
        let mut store = Store::open_at(root.join("gyrognome")).unwrap();
        let mut character =
            save::import_text(&STANDARD.encode(include_str!("fixtures/reference-save.json")))
                .unwrap();
        if !online {
            character.online = None;
        } else {
            character.online.as_mut().unwrap().host =
                "https://progressquest.com/alpaquil.php?".to_owned();
        }
        let id = store.register(&character).unwrap().id;
        Self { root, id }
    }

    fn store(&self) -> Store {
        Store::open_at(self.root.join("gyrognome")).unwrap()
    }

    fn run(&self, args: &[&str], overrides: &[(&str, &str)]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_gyrognome"))
            .args(args)
            .env("XDG_DATA_HOME", &self.root)
            .env("GYROGNOME_TEST_ACTION_LOG", self.root.join("actions"))
            .env("GYROGNOME_TEST_GUILD_OUTCOME", "accepted")
            .envs(overrides.iter().copied())
            .output()
            .unwrap()
    }

    fn actions(&self) -> Vec<String> {
        match fs::read_to_string(self.root.join("actions")) {
            Ok(text) => text.lines().map(str::to_owned).collect(),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Vec::new(),
            Err(error) => panic!("{error}"),
        }
    }
}

impl Drop for ProfileCli {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.root).unwrap();
    }
}

fn assert_safe(output: &Output) {
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    for forbidden in [
        "4242",
        "unrecognized-future-field",
        "alpaquil.php?",
        "Type yes",
        "passkey",
    ] {
        assert!(!text.contains(forbidden), "{text}");
    }
}

#[test]
fn profile_cli_submits_once_without_confirmation_with_active_or_inactive_worker() {
    for active in [false, true] {
        let cli = ProfileCli::new(true);
        let _worker = active.then(|| Worker::start(cli.store(), cli.id.clone()).unwrap());
        let id = cli.id.to_string();
        for args in [
            vec!["motto", &id, "Onward \u{2603}"],
            vec!["motto", &id, "--clear"],
            vec!["guild", &id, "Test Guild"],
            vec!["guild", &id, ""],
        ] {
            let output = cli.run(&args, &[]);
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            assert_safe(&output);
            let profile = cli.store().get(&cli.id).unwrap().state.profile;
            match args[0] {
                "motto" => assert_eq!(
                    profile.motto,
                    if args[2] == "--clear" { "" } else { args[2] }
                ),
                "guild" => assert_eq!(profile.guild, args[2]),
                _ => unreachable!(),
            }
        }
        assert_eq!(cli.actions(), ["report", "report", "guild", "guild"]);
    }
}

#[test]
fn profile_cli_retains_motto_on_failed_delivery_and_guild_on_unaccepted_result() {
    let cli = ProfileCli::new(true);
    let id = cli.id.to_string();
    for delivery in ["rejected", "failed"] {
        let output = cli.run(
            &["motto", &id, "Still saved"],
            &[("GYROGNOME_TEST_REPORT", delivery)],
        );
        assert!(output.status.success());
        assert_safe(&output);
        assert!(String::from_utf8_lossy(&output.stdout).contains("retained"));
        assert_eq!(
            cli.store().get(&cli.id).unwrap().state.profile.motto,
            "Still saved"
        );
    }
    assert!(cli.run(&["guild", &id, "Original"], &[]).status.success());
    for outcome in ["rejected", "indeterminate"] {
        let output = cli.run(
            &["guild", &id, ""],
            &[("GYROGNOME_TEST_GUILD_OUTCOME", outcome)],
        );
        assert!(output.status.success());
        assert_safe(&output);
        assert_eq!(
            cli.store().get(&cli.id).unwrap().state.profile.guild,
            "Original"
        );
    }
    assert_eq!(
        cli.actions(),
        ["report", "report", "guild", "guild", "guild"]
    );
}

#[test]
fn profile_cli_invalid_arguments_evidence_and_offline_characters_never_send() {
    let cli = ProfileCli::new(true);
    let id = cli.id.to_string();
    for args in [
        vec!["motto", &id],
        vec!["motto", &id, "text", "--clear"],
        vec!["guild", &id],
        vec!["guild", "not-an-id", "Test"],
        vec!["motto", "not-an-id", "--clear"],
        vec!["motto", &id, "bad\ntext"],
        vec!["guild", &id, "bad\u{85}text"],
    ] {
        let output = cli.run(&args, &[]);
        assert!(!output.status.success(), "{args:?}");
        assert_safe(&output);
    }
    let output = cli.run(
        &["guild", &id, "Test"],
        &[("GYROGNOME_TEST_GUILD_EVIDENCE", "invalid")],
    );
    assert!(!output.status.success());
    assert_safe(&output);
    assert!(cli.actions().is_empty());
    let offline = ProfileCli::new(false);
    for action in ["motto", "guild"] {
        let output = offline.run(&[action, &offline.id.to_string(), "Test"], &[]);
        assert!(!output.status.success());
        assert_safe(&output);
    }
    assert!(offline.actions().is_empty());
}
