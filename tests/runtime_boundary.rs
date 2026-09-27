use std::{fs, process::Command};

#[test]
fn reporting_is_the_only_http_transport_execution_path() {
    let manifest = fs::read_to_string("Cargo.toml").unwrap();
    for dependency in ["reqwest", "hyper", "curl", "isahc"] {
        assert!(
            !manifest.contains(dependency),
            "reporting must use only the dedicated HTTPS client"
        );
    }

    for source in [
        "src/runtime.rs",
        "src/lifecycle.rs",
        "src/cli.rs",
        "src/simulation.rs",
        "src/protocol.rs",
        "src/conformance_bridge.rs",
        "src/newguy.rs",
        "src/newguy_wizard.rs",
    ] {
        let content = fs::read_to_string(source).unwrap();
        for forbidden in ["std::net", "reqwest", "ureq", "hyper::", "TcpStream"] {
            assert!(
                !content.contains(forbidden),
                "{source} must not execute network transport through {forbidden}"
            );
        }
    }
    let reporting = fs::read_to_string("src/reporting.rs").unwrap();
    assert!(reporting.contains("ureq::get"));
    assert!(reporting.contains("OFFICIAL_LEADERBOARD_ENDPOINT"));
}

#[test]
fn normal_cli_build_does_not_expose_the_test_only_browser_bridge() {
    let manifest = fs::read_to_string("Cargo.toml").unwrap();
    let cli = fs::read_to_string("src/cli.rs").unwrap();

    assert!(manifest.contains("conformance-bridge = []"));
    assert!(cli.contains("#[cfg(feature = \"conformance-bridge\")]"));
    assert!(
        !cli.contains("ureq::"),
        "only the reporting service may initiate HTTP"
    );
    for source in ["src/newguy.rs", "src/newguy_wizard.rs"] {
        let content = fs::read_to_string(source).unwrap();
        for forbidden in [
            "ReportEvent",
            "ReportTrigger",
            "explicit_report_events",
            "passkey",
        ] {
            assert!(
                !content.contains(forbidden),
                "{source} must not construct leaderboard requests or credentials"
            );
        }
    }

    let help = Command::new(env!("CARGO_BIN_EXE_gyro"))
        .arg("--help")
        .output()
        .unwrap();
    assert!(help.status.success());
    let help = String::from_utf8(help.stdout).unwrap();
    assert!(help.starts_with("Offline Progress Quest compatibility tools\n\nUsage: gyro "));
    assert!(!help.contains("conformance-bridge"));
}

#[test]
fn legacy_executable_remains_compatible_with_installed_user_services() {
    let help = Command::new(env!("CARGO_BIN_EXE_gyrognome"))
        .arg("--help")
        .output()
        .unwrap();
    assert!(help.status.success());
    assert!(
        String::from_utf8(help.stdout)
            .unwrap()
            .starts_with("Offline Progress Quest compatibility tools\n\nUsage: gyrognome ")
    );
}

#[test]
fn enrollment_activation_is_limited_to_the_interactive_new_guy_command() {
    for source in [
        "src/newguy.rs",
        "src/runtime.rs",
        "src/lifecycle.rs",
        "src/dashboard.rs",
        "src/save.rs",
        "src/simulation.rs",
        "src/conformance_bridge.rs",
    ] {
        let content = fs::read_to_string(source).unwrap();
        assert!(
            !content.contains("reporting::enroll"),
            "{source} must not activate online enrollment"
        );
    }
    let cli = fs::read_to_string("src/cli.rs").unwrap();
    assert_eq!(
        cli.matches("reporting::enroll").count(),
        3,
        "only the interactive new-guy command may select enrollment"
    );
}

#[test]
fn dashboard_presentation_cannot_access_raw_save_documents() {
    let dashboard = fs::read_to_string("src/dashboard.rs").unwrap();
    for forbidden in [".document", "original_document", "passkey"] {
        assert!(
            !dashboard.contains(forbidden),
            "dashboard presentation must not access {forbidden}"
        );
    }
}

#[test]
fn packaged_user_service_is_rootless_and_runs_the_local_worker() {
    let unit = fs::read_to_string("systemd/user/gyrognome@.service").unwrap();
    assert!(unit.contains("ExecStart=gyro worker %i"));
    assert!(!unit.contains("User=root"));
    assert!(!unit.contains("sudo"));
}
