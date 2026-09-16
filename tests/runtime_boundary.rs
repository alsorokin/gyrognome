use std::{fs, process::Command};

#[test]
fn local_runtime_has_no_http_transport_dependency_or_execution_path() {
    let manifest = fs::read_to_string("Cargo.toml").unwrap();
    for dependency in ["reqwest", "ureq", "hyper", "curl", "isahc"] {
        assert!(
            !manifest.contains(dependency),
            "local runtime must not depend on {dependency}"
        );
    }

    for source in [
        "src/runtime.rs",
        "src/lifecycle.rs",
        "src/cli.rs",
        "src/simulation.rs",
        "src/protocol.rs",
        "src/conformance_bridge.rs",
    ] {
        let content = fs::read_to_string(source).unwrap();
        for forbidden in ["std::net", "reqwest", "ureq", "hyper::", "TcpStream"] {
            assert!(
                !content.contains(forbidden),
                "{source} must not execute network transport through {forbidden}"
            );
        }
    }
}

#[test]
fn normal_cli_build_does_not_expose_the_test_only_browser_bridge() {
    let manifest = fs::read_to_string("Cargo.toml").unwrap();
    let cli = fs::read_to_string("src/cli.rs").unwrap();

    assert!(manifest.contains("conformance-bridge = []"));
    assert!(cli.contains("#[cfg(feature = \"conformance-bridge\")]"));
    assert!(
        !cli.contains("std::net"),
        "normal CLI commands must not initiate HTTP"
    );

    let help = Command::new(env!("CARGO_BIN_EXE_gyrognome"))
        .arg("--help")
        .output()
        .unwrap();
    assert!(help.status.success());
    assert!(
        !String::from_utf8(help.stdout)
            .unwrap()
            .contains("conformance-bridge")
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
    assert!(unit.contains("ExecStart=gyrognome worker %i"));
    assert!(!unit.contains("User=root"));
    assert!(!unit.contains("sudo"));
}
