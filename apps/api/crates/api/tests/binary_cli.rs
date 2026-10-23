//! Contrato de la linea de ordenes del binario.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "tests fail loudly on purpose; a panic here is the assertion"
)]
#![allow(
    unused_crate_dependencies,
    reason = "the lint runs per target and an integration test never uses every dependency"
)]

mod support;

use std::net::TcpListener;

use support::process::{self, UNREACHABLE_DATABASE};

#[test]
fn the_openapi_flag_writes_the_committed_document_without_a_database() {
    let directory = tempfile::tempdir().expect("a temporary directory");
    let target = directory.path().join("openapi.json");

    // La base apunta a un puerto cerrado: si el flag tocara la base, fallaria.
    let output = process::command()
        .env("GIT_RUDDER__DATABASE__URL", UNREACHABLE_DATABASE)
        .args(["--openapi", target.to_str().expect("utf-8 path")])
        .output()
        .expect("the binary should start");

    assert!(output.status.success(), "--openapi failed: {output:?}");
    let written = std::fs::read_to_string(&target).expect("the document was written");
    let committed = std::fs::read_to_string(support::api_root().join("openapi.json"))
        .expect("apps/api/openapi.json should exist");
    assert!(
        written == committed,
        "apps/api/openapi.json is stale: run `just api-openapi` and commit it"
    );
}

#[test]
fn the_openapi_flag_reports_a_path_it_cannot_write() {
    let directory = tempfile::tempdir().expect("a temporary directory");
    let target = directory.path().join("missing").join("openapi.json");

    let output = process::run(&["--openapi", target.to_str().expect("utf-8 path")]);

    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn help_prints_the_usage_and_succeeds() {
    let output = process::run(&["--help"]);

    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("--openapi <path>"));
}

#[test]
fn an_unknown_argument_exits_with_2_and_the_usage() {
    let output = process::run(&["--port", "9000"]);

    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("unexpected argument \"--port\""));
    assert!(stderr.contains("usage:"));
}

#[test]
fn an_invalid_configuration_stops_the_start() {
    let output = process::command()
        .env("GIT_RUDDER__SERVER__PORT", "not-a-port")
        .output()
        .expect("the binary should start");

    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn a_port_already_in_use_stops_the_start() {
    let taken = TcpListener::bind("127.0.0.1:0").expect("a free port");
    let port = taken.local_addr().expect("a local address").port();

    let output = process::command()
        .env("GIT_RUDDER__SERVER__HOST", "127.0.0.1")
        .env("GIT_RUDDER__SERVER__PORT", port.to_string())
        .env("GIT_RUDDER__DATABASE__URL", UNREACHABLE_DATABASE)
        .output()
        .expect("the binary should start");

    assert_eq!(output.status.code(), Some(1));
    drop(taken);
}
