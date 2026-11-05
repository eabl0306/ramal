//! Un repositorio de cien mil commits abre su primera ventana en menos de un
//! segundo, y bajar no vuelve a leer lo ya leido (E05-02).

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

use std::time::{Duration, Instant};

use fixture::{FixtureRepo, path_text};
use git_rudder_engine::{CallContext, EngineBuilder, EngineConfig};
use serde_json::json;

const COMMITS: usize = 100_000;

#[tokio::test]
async fn the_first_window_of_a_hundred_thousand_commits_takes_under_a_second() {
    let repo = FixtureRepo::new();
    repo.fast_import_linear(COMMITS);
    let engine = EngineBuilder::new(EngineConfig::default()).build();
    let handle = engine
        .dispatch(
            "repo/open",
            json!({ "path": path_text(&repo.path()) }),
            CallContext::host(),
        )
        .await
        .unwrap();
    let repo_id = handle["repoId"].clone();

    let started = Instant::now();
    let first = engine
        .dispatch(
            "graph/window",
            json!({ "repoId": repo_id, "offset": 0, "limit": 200 }),
            CallContext::webview(),
        )
        .await
        .unwrap();
    let elapsed = started.elapsed();

    assert_eq!(first["rows"].as_array().unwrap().len(), 200);
    assert_eq!(first["hasMore"], true);
    assert!(
        elapsed < Duration::from_secs(1),
        "first window took {elapsed:?}"
    );

    let started = Instant::now();
    let next = engine
        .dispatch(
            "graph/window",
            json!({ "repoId": repo_id, "offset": 200, "limit": 200, "snapshot": first["snapshot"] }),
            CallContext::webview(),
        )
        .await
        .unwrap();
    assert_eq!(next["offset"], 200);
    assert_eq!(
        next["rows"][0]["summary"], "commit 9800",
        "row 200 is the commit 100000 - 200"
    );
    assert!(
        started.elapsed() < elapsed.max(Duration::from_millis(50)),
        "the next page comes from the cache: {:?}",
        started.elapsed()
    );
}
