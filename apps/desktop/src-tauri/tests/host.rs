//! Los comandos del host por IPC sobre el runtime simulado de Tauri
//! (`tauri::test`): estan registrados, serializan como espera el frontend,
//! respetan las capacidades compiladas y la lista de metodos de la ventana, y
//! abren de verdad un repositorio creado con git en un tempdir.

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

#[path = "../../../../crates/git/tests/support/mod.rs"]
mod fixture;

use std::sync::Arc;

use fixture::FixtureRepo;
use git_rudder_desktop::bootstrap::{build_engine, configure};
use git_rudder_desktop::picker::{FixedFolderPicker, FolderPicker};
use pretty_assertions::assert_eq;
use serde_json::{Value, json};
use tauri::ipc::{CallbackFn, InvokeBody};
use tauri::test::{INVOKE_KEY, MockRuntime, get_ipc_response, mock_builder};
use tauri::webview::InvokeRequest;
use tauri::{App, WebviewWindow, WebviewWindowBuilder};

/// La app con los comandos reales, el contexto real (configuracion y
/// capacidades) y un selector de carpetas fijo en lugar del dialogo.
fn app_with(picker: FixedFolderPicker) -> App<MockRuntime> {
    let picker: Arc<dyn FolderPicker> = Arc::new(picker);
    configure(mock_builder(), Arc::new(build_engine()))
        .manage(picker)
        .build(git_rudder_desktop::context())
        .expect("the app must build with the mock runtime")
}

/// La ventana principal tal como la declara `tauri.conf.json`. La app real la crea
/// al arrancar el bucle de eventos, que los tests no arrancan.
fn main_window(app: &App<MockRuntime>) -> WebviewWindow<MockRuntime> {
    let config = app
        .config()
        .app
        .windows
        .iter()
        .find(|window| window.label == "main")
        .expect("tauri.conf.json declares the main window")
        .clone();
    WebviewWindowBuilder::from_config(app.handle(), &config)
        .and_then(WebviewWindowBuilder::build)
        .expect("the main window can be created")
}

fn invoke(window: &WebviewWindow<MockRuntime>, command: &str, body: Value) -> Result<Value, Value> {
    let request = InvokeRequest {
        cmd: command.into(),
        callback: CallbackFn(0),
        error: CallbackFn(1),
        url: "tauri://localhost".parse().unwrap(),
        body: InvokeBody::Json(body),
        headers: tauri::http::HeaderMap::default(),
        invoke_key: INVOKE_KEY.to_owned(),
    };
    get_ipc_response(window, request).map(|body| body.deserialize::<Value>().unwrap())
}

fn rpc(window: &WebviewWindow<MockRuntime>, id: u32, method: &str, params: &Value) -> Value {
    let request = json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params });
    invoke(window, "rpc", json!({ "request": request })).expect("rpc never rejects")
}

fn repo_with_history() -> FixtureRepo {
    let repo = FixtureRepo::new();
    repo.commit("Primer commit");
    repo.commit("Segundo commit");
    repo.commit("Tercer commit");
    repo
}

#[test]
fn the_picked_repository_is_opened_and_its_graph_reaches_the_main_window() {
    let repo = repo_with_history();
    let app = app_with(FixedFolderPicker::choosing(repo.path()));
    let window = main_window(&app);

    let handle = invoke(&window, "repo_pick_and_open", json!({})).expect("the repo opens");
    assert_eq!(handle["headRef"], "main");
    assert_eq!(handle["isEmpty"], false);
    let repo_id = handle["repoId"]
        .as_str()
        .expect("the handle carries a repoId");

    let response = rpc(
        &window,
        7,
        "graph/window",
        &json!({ "repoId": repo_id, "offset": 0, "limit": 50 }),
    );
    assert_eq!(response["id"], 7);
    let summaries: Vec<&str> = response["result"]["rows"]
        .as_array()
        .expect("the window has rows")
        .iter()
        .map(|row| row["summary"].as_str().unwrap())
        .collect();
    assert_eq!(
        summaries,
        ["Tercer commit", "Segundo commit", "Primer commit"]
    );
}

#[test]
fn cancelling_the_picker_answers_null() {
    let app = app_with(FixedFolderPicker::cancelling());
    let window = main_window(&app);

    assert_eq!(
        invoke(&window, "repo_pick_and_open", json!({})),
        Ok(Value::Null)
    );
}

#[test]
fn a_folder_outside_any_repository_rejects_with_the_engine_kind() {
    let repo = FixtureRepo::new();
    let app = app_with(FixedFolderPicker::choosing(repo.outside()));
    let window = main_window(&app);

    let error = invoke(&window, "repo_pick_and_open", json!({})).expect_err("not a repo");
    assert_eq!(error["data"]["kind"], "not_a_repository");
    assert_eq!(error["code"], -32_000);
}

#[test]
fn the_webview_cannot_open_a_path_through_rpc() {
    let repo = repo_with_history();
    let app = app_with(FixedFolderPicker::cancelling());
    let window = main_window(&app);

    let response = rpc(
        &window,
        1,
        "repo/open",
        &json!({ "path": fixture::path_text(&repo.path()) }),
    );
    assert_eq!(response["error"]["code"], -32_601);
    assert_eq!(response["error"]["data"]["kind"], "internal");
    assert_eq!(response.get("result"), None);
}

#[test]
fn a_method_outside_the_window_list_is_refused() {
    let app = app_with(FixedFolderPicker::cancelling());
    let window = main_window(&app);

    let response = rpc(&window, 2, "engine/info", &json!({}));
    assert_eq!(response["error"]["code"], -32_601);
}

#[test]
fn an_unknown_repo_id_comes_back_as_an_engine_error() {
    let app = app_with(FixedFolderPicker::cancelling());
    let window = main_window(&app);

    let response = rpc(
        &window,
        3,
        "graph/window",
        &json!({ "repoId": "repo-9", "offset": 0, "limit": 10 }),
    );
    assert_eq!(response["error"]["data"]["kind"], "repo_not_open");
}

#[test]
fn notifications_get_no_response() {
    let app = app_with(FixedFolderPicker::cancelling());
    let window = main_window(&app);

    for method in ["$/cancel", "graph/window"] {
        let notification = json!({ "jsonrpc": "2.0", "method": method, "params": { "id": 4 } });
        assert_eq!(
            invoke(&window, "rpc", json!({ "request": notification })),
            Ok(Value::Null)
        );
    }
}

#[test]
fn a_malformed_envelope_is_rejected_by_the_transport() {
    let app = app_with(FixedFolderPicker::cancelling());
    let window = main_window(&app);

    let too_big = json!({ "jsonrpc": "2.0", "id": 2_147_483_648_u64, "method": "graph/window" });
    assert!(invoke(&window, "rpc", json!({ "request": too_big })).is_err());
}

#[test]
fn a_window_without_the_capability_cannot_invoke_the_commands() {
    let app = app_with(FixedFolderPicker::cancelling());
    let other = WebviewWindowBuilder::new(&app, "other", tauri::WebviewUrl::default())
        .build()
        .expect("a second window can be created");

    assert!(invoke(&other, "repo_pick_and_open", json!({})).is_err());
    let request = json!({ "jsonrpc": "2.0", "id": 1, "method": "graph/window", "params": {} });
    assert!(invoke(&other, "rpc", json!({ "request": request })).is_err());
}
