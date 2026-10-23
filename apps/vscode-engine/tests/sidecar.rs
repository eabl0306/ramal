//! El binario real por stdio: marcos `Content-Length`, peticiones en paralelo,
//! errores del sobre, marco demasiado grande y cierre al cerrar stdin.

#![allow(
    unused_crate_dependencies,
    reason = "the lint runs per target and this test does not use every dependency"
)]
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "tests fail loudly on purpose; a panic here is a failed assertion"
)]

#[path = "../../../crates/git/tests/support/mod.rs"]
mod fixture;

use std::collections::BTreeSet;
use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, ExitStatus, Stdio};

use fixture::{FixtureRepo, path_text};
use pretty_assertions::assert_eq;
use ramal_protocol::{PROTOCOL_VERSION, error_code};
use serde_json::{Value, json};

const BINARY: &str = env!("CARGO_BIN_EXE_ramal-vscode-engine");

struct Sidecar {
    child: Child,
    stdin: Option<ChildStdin>,
    stdout: BufReader<ChildStdout>,
}

impl Sidecar {
    fn start(args: &[&str]) -> Self {
        let mut child = Command::new(BINARY)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("the sidecar binary should start");
        let stdin = child.stdin.take();
        let stdout = BufReader::new(child.stdout.take().unwrap());
        Self {
            child,
            stdin,
            stdout,
        }
    }

    fn send_raw(&mut self, bytes: &[u8]) {
        let stdin = self.stdin.as_mut().unwrap();
        stdin.write_all(bytes).unwrap();
        stdin.flush().unwrap();
    }

    fn send(&mut self, body: &[u8]) {
        self.send_raw(format!("Content-Length: {}\r\n\r\n", body.len()).as_bytes());
        self.send_raw(body);
    }

    fn send_request(&mut self, id: u32, method: &str, params: &Value) {
        let request = json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params });
        self.send(&serde_json::to_vec(&request).unwrap());
    }

    fn receive(&mut self) -> Value {
        let mut length = None;
        loop {
            let mut line = String::new();
            assert!(
                self.stdout.read_line(&mut line).unwrap() > 0,
                "stdout closed"
            );
            let line = line.trim_end();
            if line.is_empty() {
                break;
            }
            if let Some(value) = line.strip_prefix("Content-Length: ") {
                length = Some(value.parse::<usize>().unwrap());
            }
        }
        let mut body = vec![0; length.expect("a frame always has Content-Length")];
        self.stdout.read_exact(&mut body).unwrap();
        serde_json::from_slice(&body).unwrap()
    }

    fn request(&mut self, id: u32, method: &str, params: &Value) -> Value {
        self.send_request(id, method, params);
        let response = self.receive();
        assert_eq!(response["id"], json!(id));
        response
    }

    fn finish(mut self) -> ExitStatus {
        drop(self.stdin.take());
        self.child.wait().unwrap()
    }
}

#[test]
fn it_answers_info_open_and_graph_window_over_stdio() {
    let repo = FixtureRepo::with_merged_feature();
    let mut sidecar = Sidecar::start(&["--log-level", "debug"]);

    let info = sidecar.request(1, "engine/info", &json!({}));
    assert_eq!(info["result"]["protocolVersion"], json!(PROTOCOL_VERSION));
    assert_eq!(info["result"]["git"]["supported"], json!(true));

    let open = sidecar.request(2, "repo/open", &json!({ "path": path_text(&repo.path()) }));
    let repo_id = open["result"]["repoId"].as_str().unwrap().to_owned();
    assert_eq!(open["result"]["headRef"], json!("main"));

    let params = json!({ "repoId": repo_id, "offset": 0, "limit": 200 });
    let window = sidecar.request(3, "graph/window", &params);
    let rows = window["result"]["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 4);
    assert_eq!(rows[0]["isHead"], json!(true));
    assert_eq!(rows[0]["refs"][0]["name"], json!("main"));
    assert_eq!(window["result"]["laneCount"], json!(2));

    assert!(sidecar.finish().success());
}

#[test]
fn requests_sent_together_all_get_their_answer() {
    let mut sidecar = Sidecar::start(&[]);

    for id in [10, 11, 12] {
        sidecar.send_request(id, "engine/info", &json!({}));
    }
    let ids: BTreeSet<u64> = (0..3)
        .map(|_| sidecar.receive()["id"].as_u64().unwrap())
        .collect();

    assert_eq!(ids, BTreeSet::from([10, 11, 12]));
    assert!(sidecar.finish().success());
}

#[test]
fn broken_json_gets_a_parse_error_and_the_connection_stays_open() {
    let mut sidecar = Sidecar::start(&[]);

    sidecar.send(b"{\"jsonrpc\":");
    let error = sidecar.receive();
    assert_eq!(error["id"], Value::Null);
    assert_eq!(error["error"]["code"], json!(error_code::PARSE_ERROR));

    let info = sidecar.request(4, "engine/info", &json!({}));
    assert!(info.get("result").is_some());
    assert!(sidecar.finish().success());
}

#[test]
fn a_frame_over_32_mib_closes_the_connection() {
    let mut sidecar = Sidecar::start(&[]);

    sidecar.send_raw(b"Content-Length: 40000000\r\n\r\n");
    let status = sidecar.child.wait().unwrap();

    assert_eq!(status.code(), Some(1));
}

#[test]
fn a_missing_git_is_reported_as_git_not_found() {
    let mut sidecar = Sidecar::start(&["--git-path", "/no/such/git"]);

    let response = sidecar.request(1, "engine/info", &json!({}));

    assert_eq!(response["error"]["data"]["kind"], json!("git_not_found"));
    assert!(sidecar.finish().success());
}

#[test]
fn the_version_flag_prints_the_version() {
    let output = Command::new(BINARY).arg("--version").output().unwrap();

    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).unwrap();
    assert_eq!(
        text,
        format!("ramal-vscode-engine {}\n", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn an_unknown_argument_exits_with_the_usage() {
    let output = Command::new(BINARY).arg("--stdio").output().unwrap();

    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("usage:"));
    assert!(output.stdout.is_empty(), "{:?}", output.stdout);
}
