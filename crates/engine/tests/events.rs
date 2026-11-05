//! `$/cancel` y la notificacion `repo/changed` de punta a punta.

#![allow(
    unused_crate_dependencies,
    reason = "the lint runs per target and this test does not use every dependency"
)]
#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail loudly on purpose; a panic here is a failed assertion"
)]

#[path = "../../git/tests/support/mod.rs"]
mod fixture;

use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use fixture::{FixtureRepo, path_text};
use git_rudder_core_application::testing::FakeGitReader;
use git_rudder_core_application::{GitReadError, GitReader, GitVersionInfo, LogQuery};
use git_rudder_core_domain::{CommitRecord, RefSnapshot, RepoRoot};
use git_rudder_engine::{CallContext, EngineBuilder, EngineConfig, NotificationSink};
use git_rudder_protocol::{
    ErrorKind, RequestId, RpcNotification, RpcOutcome, RpcRequest, error_code,
};
use serde_json::json;
use tokio::sync::mpsc;

/// Un git que nunca termina de leer refs: la peticion queda en curso.
struct HangingGit(FakeGitReader);

#[async_trait]
impl GitReader for HangingGit {
    async fn version(&self) -> Result<GitVersionInfo, GitReadError> {
        self.0.version().await
    }

    async fn discover(&self, path: &Path) -> Result<RepoRoot, GitReadError> {
        self.0.discover(path).await
    }

    async fn read_refs(&self, _repo: &RepoRoot) -> Result<RefSnapshot, GitReadError> {
        std::future::pending().await
    }

    async fn read_log(
        &self,
        repo: &RepoRoot,
        query: &LogQuery,
    ) -> Result<Vec<CommitRecord>, GitReadError> {
        self.0.read_log(repo, query).await
    }
}

fn request(id: Option<u32>, method: &str, params: serde_json::Value) -> RpcRequest {
    serde_json::from_value(json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": method,
        "params": params,
    }))
    .unwrap()
}

#[tokio::test]
async fn a_cancelled_request_answers_cancelled_with_the_lsp_code() {
    let fake = FakeGitReader::new().with_repository("/work/r", RepoRoot::new("/work/r"));
    let engine = Arc::new(
        EngineBuilder::new(EngineConfig::default())
            .with_git_reader(Arc::new(HangingGit(fake)))
            .build(),
    );
    // repo/open lee las refs despues de descubrir la raiz: con este git se queda
    // esperando, como un git lento de verdad.
    let open = request(
        Some(9),
        "repo/open",
        json!({ "path": "/work/r", "trusted": true }),
    );
    let pending = tokio::spawn({
        let engine = Arc::clone(&engine);
        async move { engine.handle_with(open, CallContext::host()).await }
    });
    tokio::time::sleep(Duration::from_millis(50)).await;

    let ignored = engine
        .handle(request(None, "$/cancel", json!({ "id": 9 })))
        .await;
    let response = tokio::time::timeout(Duration::from_secs(5), pending)
        .await
        .unwrap()
        .unwrap()
        .unwrap();

    assert_eq!(ignored, None, "a notification gets no response");
    assert_eq!(response.id, RequestId::new(9));
    let RpcOutcome::Error(error) = response.outcome else {
        unreachable!("a cancelled call is an error: {:?}", response.outcome)
    };
    assert_eq!(error.code, error_code::REQUEST_CANCELLED);
    assert_eq!(error.data.unwrap().kind, ErrorKind::Cancelled);
}

#[tokio::test]
async fn a_cancel_for_an_unknown_id_or_with_bad_params_is_ignored() {
    let engine = EngineBuilder::new(EngineConfig::default()).build();

    assert_eq!(
        engine
            .handle(request(None, "$/cancel", json!({ "id": 1 })))
            .await,
        None
    );
    assert_eq!(
        engine
            .handle(request(None, "$/cancel", json!({ "x": 1 })))
            .await,
        None
    );
    assert_eq!(
        engine
            .handle(request(None, "other/notification", json!({})))
            .await,
        None
    );
}

/// Pasa cada notificacion a un canal del test.
struct ChannelSink(mpsc::UnboundedSender<RpcNotification>);

impl NotificationSink for ChannelSink {
    fn notify(&self, notification: RpcNotification) {
        let _ = self.0.send(notification);
    }
}

#[tokio::test]
async fn a_commit_from_the_terminal_is_announced_in_under_a_second() {
    let repo = FixtureRepo::new();
    repo.commit_files(&[(".gitignore", "build/\n"), ("a.txt", "a\n")], "Initial");
    let (sender, mut received) = mpsc::unbounded_channel();
    let engine = EngineBuilder::new(EngineConfig::default())
        .with_notification_sink(Arc::new(ChannelSink(sender)))
        .build();
    let handle = engine
        .dispatch(
            "repo/open",
            json!({ "path": path_text(&repo.path()) }),
            CallContext::host(),
        )
        .await
        .unwrap();
    // Lo que ignora .gitignore no avisa.
    repo.write("build/output.o", "binary\n");
    let quiet = tokio::time::timeout(Duration::from_millis(900), received.recv()).await;
    assert!(quiet.is_err(), "an ignored file changed: {quiet:?}");

    let started = Instant::now();
    repo.commit_files(&[("a.txt", "b\n")], "From the terminal");
    let notification = tokio::time::timeout(Duration::from_secs(3), received.recv())
        .await
        .unwrap()
        .unwrap();
    let elapsed = started.elapsed();

    assert_eq!(notification.method, "repo/changed");
    assert_eq!(notification.params, json!({ "repoId": handle["repoId"] }));
    assert!(elapsed < Duration::from_secs(1), "took {elapsed:?}");
    // La rafaga del commit sale como un solo aviso.
    tokio::time::sleep(Duration::from_millis(700)).await;
    let extra = std::iter::from_fn(|| received.try_recv().ok()).count();
    assert!(extra <= 1, "{extra} more notifications for one commit");
}
