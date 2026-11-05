//! Repos de prueba dorados (E05-22): historias dificiles construidas con git real
//! en un tempdir, y la respuesta de cada lectura guardada como snapshot en
//! `tests/snapshots/`. Un cambio en un parser o en un DTO cambia el fichero, y se
//! revisa en el diff del PR.
//!
//! Corre en Linux, macOS y Windows en la CI (`engine.yml`). Para reescribir los
//! snapshots: `GIT_RUDDER_UPDATE_SNAPSHOTS=1 cargo test -p git-rudder-engine --test golden`.

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
mod snapshot;

use fixture::{FixtureRepo, path_text};
use git_rudder_engine::{CallContext, Engine, EngineBuilder, EngineConfig};
use serde_json::{Value, json};
use snapshot::assert_snapshot;

/// Puntero de Git LFS tal cual lo commitea `git lfs` (sin tener git-lfs).
const LFS_POINTER: &str = "version https://git-lfs.github.com/spec/v1\n\
oid sha256:4d7a214614ab2935c943f9e0ff69d22eadbb8f32b1258daaa5e2ca24d17e2393\n\
size 12345\n";

/// Un repo abierto en un motor, con lo que hay que quitar de los snapshots.
struct Opened {
    engine: Engine,
    repo_id: String,
    temp_roots: Vec<String>,
}

impl Opened {
    async fn new(repo: &FixtureRepo) -> Self {
        let engine = EngineBuilder::new(EngineConfig::default()).build();
        let handle = engine
            .dispatch(
                "repo/open",
                json!({ "path": path_text(&repo.path()) }),
                CallContext::host(),
            )
            .await
            .unwrap();
        let mut temp_roots = vec![
            handle["root"].as_str().unwrap().to_owned(),
            path_text(&repo.canonical_path()),
            path_text(&repo.path()),
        ];
        // La mas larga primero: una puede contener a otra.
        temp_roots.sort_by_key(|root| std::cmp::Reverse(root.len()));
        let repo_id = handle["repoId"].as_str().unwrap().to_owned();
        Self {
            engine,
            repo_id,
            temp_roots,
        }
    }

    /// Llama como un webview, con el `repoId` ya puesto.
    async fn call(&self, method: &str, mut params: Value) -> Value {
        params["repoId"] = json!(self.repo_id);
        self.engine
            .dispatch(method, params, CallContext::webview())
            .await
            .unwrap()
    }

    async fn snapshot(&self, name: &str, method: &str, params: Value) {
        let value = self.call(method, params).await;
        assert_snapshot(name, &value, &self.temp_roots);
    }
}

/// main con un renombrado en una rama, un merge, un octopus de tres ramas, un
/// binario, un puntero LFS y un tag anotado.
fn tangled_history() -> (FixtureRepo, String, String, String) {
    let repo = FixtureRepo::new();
    repo.commit_files(
        &[
            ("README.md", "# Demo\n\nUna linea.\n"),
            ("src/old.rs", "fn main() {}\n"),
        ],
        "Initial",
    );
    repo.checkout_new("feature/rename");
    repo.rename("src/old.rs", "src/new.rs", "Rename old.rs to new.rs");
    repo.checkout("main");
    repo.commit_files(&[("README.md", "# Demo\n\nOtra linea.\n")], "Edit README");
    let merge = repo.merge(&["feature/rename"], "Merge feature/rename");
    for name in ["a", "b", "c"] {
        repo.checkout_new(name);
        repo.commit_files(
            &[(&format!("{name}.txt"), &format!("{name}\n"))],
            &format!("Add {name}"),
        );
        repo.checkout("main");
    }
    let octopus = repo.merge(&["a", "b", "c"], "Octopus a, b and c");
    repo.commit_files(
        &[
            (
                ".gitattributes",
                "*.png filter=lfs diff=lfs merge=lfs -text\n",
            ),
            ("assets/logo.png", LFS_POINTER),
        ],
        "Add logo through LFS",
    );
    repo.write_bytes("bin/data.bin", &[0, 159, 146, 150, 0, 1, 2, 3]);
    repo.add(&["bin/data.bin"]);
    let binary = repo.commit("Add binary data");
    repo.annotated_tag("v1.0", "First release");
    (repo, merge, octopus, binary)
}

#[tokio::test]
async fn a_tangled_history_reads_the_same_on_every_platform() {
    let (repo, merge, octopus, binary) = tangled_history();
    let opened = Opened::new(&repo).await;
    let commit = |sha: &str| json!({ "kind": "commit", "sha": sha });
    let logo = commit(&repo.rev("HEAD~1"));
    let reads = [
        (
            "history-graph",
            "graph/window",
            json!({ "offset": 0, "limit": 100 }),
        ),
        ("history-refs", "refs/list", json!({})),
        (
            "history-merge-detail",
            "commit/detail",
            json!({ "sha": merge }),
        ),
        (
            "history-octopus-detail",
            "commit/detail",
            json!({ "sha": octopus }),
        ),
        (
            "history-lfs-diff",
            "diff/file",
            json!({ "path": "assets/logo.png", "target": logo }),
        ),
        (
            "history-binary-diff",
            "diff/file",
            json!({ "path": "bin/data.bin", "target": commit(&binary) }),
        ),
        (
            "history-file-renamed",
            "history/file",
            json!({ "path": "src/new.rs", "offset": 0, "limit": 10 }),
        ),
        (
            "history-blame",
            "blame/file",
            json!({ "path": "README.md" }),
        ),
    ];

    for (name, method, params) in reads {
        opened.snapshot(name, method, params).await;
    }
}

#[tokio::test]
async fn submodules_read_the_same_on_every_platform() {
    let inner = FixtureRepo::new();
    inner.commit_files(&[("lib.rs", "pub fn lib() {}\n")], "Inner initial");
    let outer = FixtureRepo::new();
    outer.commit_files(&[("README.md", "outer\n")], "Outer initial");
    outer.add_submodule(&inner, "vendor/inner");
    outer.add_submodule(&inner, "vendor/pristine");
    // Uno con otro commit sacado (modificado) y otro sin inicializar.
    std::fs::write(
        outer.path().join("vendor/inner/lib.rs"),
        "pub fn changed() {}\n",
    )
    .unwrap();
    outer.tick();
    outer.git(&[
        "-C",
        "vendor/inner",
        "commit",
        "--quiet",
        "-am",
        "Change inside",
    ]);
    outer.git(&[
        "submodule",
        "deinit",
        "--quiet",
        "--force",
        "vendor/pristine",
    ]);
    let opened = Opened::new(&outer).await;

    opened
        .snapshot("submodules-list", "submodule/list", json!({}))
        .await;
    opened
        .snapshot("submodules-status", "status/read", json!({}))
        .await;
}
