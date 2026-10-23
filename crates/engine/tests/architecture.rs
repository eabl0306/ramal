//! Reglas de dependencia del hexagono del cliente (docs/architecture.md 5.1).
//!
//! Lee el `Cargo.lock` de la raiz linea a linea, sin crate de TOML, construye el
//! grafo y comprueba: que las dependencias directas de los nucleos son un
//! subconjunto de su lista blanca, que el cierre transitivo de cada crate no
//! contiene nada de su lista negra, que los hosts solo hablan con el motor y que
//! ninguna familia (cliente y API) alcanza a la otra salvo `git-rudder-ai`. Puede mirar
//! `git-rudder-desktop` sin compilar Tauri porque `Cargo.lock` describe a todos los
//! miembros del workspace. Los `clippy.toml` de los nucleos (umbrales y APIs de IO
//! prohibidas) los vigila tests/core_purity.rs.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "tests fail loudly on purpose; a panic here is the assertion"
)]
#![allow(
    unused_crate_dependencies,
    reason = "the lint runs per target and this test only reads files"
)]

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::PathBuf;

type Graph = BTreeMap<String, BTreeSet<String>>;

/// Crates del hexagono del cliente.
const CLIENT_CRATES: &[&str] = &[
    "git-rudder-core-domain",
    "git-rudder-core-application",
    "git-rudder-protocol",
    "git-rudder-git",
    "git-rudder-engine",
    "git-rudder-vscode-engine",
    "git-rudder-desktop",
];

/// Lo que ningun nucleo del cliente puede alcanzar. `serde` si entra: su cierre
/// es diminuto y ningun crate permitido lo trae.
const CORE_FORBIDDEN: &[&str] = &[
    "serde",
    "serde_json",
    "ts-rs",
    "tokio",
    "reqwest",
    "hyper",
    "sqlx",
    "salvo",
    "tauri",
    "gix",
    "git2",
    "libgit2-sys",
    "keyring-core",
];

const DOMAIN_ALLOWED: &[&str] = &["thiserror"];
const APPLICATION_ALLOWED: &[&str] = &["git-rudder-core-domain", "async-trait", "thiserror"];

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn unquote(raw: &str) -> String {
    raw.trim()
        .trim_end_matches(',')
        .trim_matches('"')
        .to_owned()
}

/// Mapa `paquete -> dependencias directas` (incluidas las de desarrollo).
fn dependency_graph(lock: &str) -> Graph {
    let mut graph = Graph::new();
    let mut current: Option<String> = None;
    let mut in_dependencies = false;
    for line in lock.lines().map(str::trim) {
        if line == "[[package]]" {
            current = None;
            in_dependencies = false;
        } else if let Some(name) = line.strip_prefix("name = ") {
            current = Some(unquote(name));
            graph.entry(unquote(name)).or_default();
        } else if line == "dependencies = [" {
            in_dependencies = true;
        } else if in_dependencies && line == "]" {
            in_dependencies = false;
        } else if in_dependencies && let Some(owner) = &current {
            let entry = unquote(line);
            let name = entry
                .split_whitespace()
                .next()
                .unwrap_or_default()
                .to_owned();
            graph.entry(owner.clone()).or_default().insert(name);
        }
    }
    graph
}

fn graph() -> Graph {
    let lock = std::fs::read_to_string(root().join("Cargo.lock"))
        .expect("Cargo.lock should exist at the workspace root");
    dependency_graph(&lock)
}

fn closure(graph: &Graph, crate_name: &str) -> BTreeSet<String> {
    let mut seen = BTreeSet::new();
    let mut queue = VecDeque::from([crate_name.to_owned()]);
    while let Some(package) = queue.pop_front() {
        for dependency in graph.get(&package).into_iter().flatten() {
            if seen.insert(dependency.clone()) {
                queue.push_back(dependency.clone());
            }
        }
    }
    seen
}

fn direct<'a>(graph: &'a Graph, crate_name: &str) -> &'a BTreeSet<String> {
    graph
        .get(crate_name)
        .unwrap_or_else(|| unreachable!("{crate_name} should be in Cargo.lock"))
}

fn assert_never_reaches(graph: &Graph, crate_name: &str, forbidden: impl Fn(&str) -> bool) {
    for reached in closure(graph, crate_name) {
        assert!(
            !forbidden(&reached),
            "{crate_name} reaches `{reached}`. The hexagon leaks: move that code to an adapter"
        );
    }
}

fn is_tauri(name: &str) -> bool {
    name == "tauri" || name.starts_with("tauri-")
}

fn is_api(name: &str) -> bool {
    name.starts_with("git-rudder-api")
}

#[test]
fn the_lock_file_describes_every_client_crate() {
    let graph = graph();
    for crate_name in CLIENT_CRATES {
        assert!(
            graph.contains_key(*crate_name),
            "{crate_name} is missing from Cargo.lock"
        );
    }
}

#[test]
fn the_parser_sees_real_edges() {
    let graph = graph();
    assert!(closure(&graph, "git-rudder-git").contains("tokio"));
    assert!(closure(&graph, "git-rudder-protocol").contains("serde"));
    assert!(closure(&graph, "git-rudder-protocol").contains("ts-rs"));
    assert!(direct(&graph, "git-rudder-engine").contains("git-rudder-git"));
    assert!(direct(&graph, "git-rudder-vscode-engine").contains("git-rudder-engine"));
    assert!(closure(&graph, "git-rudder-vscode-engine").contains("git-rudder-core-domain"));
}

#[test]
fn the_core_only_declares_its_allow_list() {
    let graph = graph();
    for (crate_name, allowed) in [
        ("git-rudder-core-domain", DOMAIN_ALLOWED),
        ("git-rudder-core-application", APPLICATION_ALLOWED),
    ] {
        for dependency in direct(&graph, crate_name) {
            assert!(
                allowed.contains(&dependency.as_str()),
                "{crate_name} declares `{dependency}`; allowed: {allowed:?}. \
                 The hexagon leaks: move that code to an adapter"
            );
        }
    }
}

#[test]
fn the_core_never_reaches_infrastructure() {
    let graph = graph();
    assert_never_reaches(&graph, "git-rudder-core-domain", |name| {
        CORE_FORBIDDEN.contains(&name) || name == "async-trait" || name.starts_with("git-rudder-")
    });
    assert_never_reaches(&graph, "git-rudder-core-application", |name| {
        CORE_FORBIDDEN.contains(&name)
            || (name.starts_with("git-rudder-") && name != "git-rudder-core-domain")
    });
}

#[test]
fn the_protocol_does_not_know_the_engine() {
    let graph = graph();
    assert_never_reaches(&graph, "git-rudder-protocol", |name| {
        name.starts_with("git-rudder-")
            || ["tokio", "salvo", "sqlx"].contains(&name)
            || is_tauri(name)
    });
}

#[test]
fn the_git_adapter_does_not_know_the_protocol_or_the_hosts() {
    let graph = graph();
    let forbidden = [
        "git-rudder-protocol",
        "git-rudder-engine",
        "git-rudder-desktop",
        "git-rudder-vscode-engine",
    ];
    assert_never_reaches(&graph, "git-rudder-git", |name| {
        forbidden.contains(&name)
            || ["salvo", "sqlx", "git2", "libgit2-sys"].contains(&name)
            || is_tauri(name)
    });
}

#[test]
fn the_engine_stays_out_of_hosts_and_the_server() {
    let graph = graph();
    assert_never_reaches(&graph, "git-rudder-engine", |name| {
        is_tauri(name) || is_api(name) || ["salvo", "sqlx", "git2", "libgit2-sys"].contains(&name)
    });
}

#[test]
fn the_hosts_only_talk_to_the_engine() {
    let graph = graph();
    for host in ["git-rudder-vscode-engine", "git-rudder-desktop"] {
        for dependency in direct(&graph, host) {
            assert!(
                !dependency.starts_with("git-rudder-core-") && dependency != "git-rudder-git",
                "{host} declares `{dependency}`: the host talks to the engine, not to its parts"
            );
        }
        assert_never_reaches(&graph, host, |name| {
            ["salvo", "sqlx"].contains(&name) || is_api(name)
        });
    }
    assert_never_reaches(&graph, "git-rudder-vscode-engine", is_tauri);
}

#[test]
fn the_client_and_the_server_never_reach_each_other() {
    let graph = graph();
    for crate_name in CLIENT_CRATES {
        assert_never_reaches(&graph, crate_name, is_api);
    }
    for api_crate in graph.keys().filter(|name| is_api(name)) {
        assert_never_reaches(&graph, api_crate, |name| CLIENT_CRATES.contains(&name));
    }
    if graph.contains_key("git-rudder-ai") {
        assert_never_reaches(&graph, "git-rudder-ai", |name| {
            name.starts_with("git-rudder-")
        });
    }
}
