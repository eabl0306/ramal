//! Test de arquitectura del hexagono del servidor.
//!
//! Lee el `Cargo.lock` de la raiz linea a linea, sin crate de TOML, construye el
//! grafo de dependencias y comprueba las reglas de la seccion 5.1 de
//! docs/architecture.md: lista blanca directa en el nucleo, lista negra
//! transitiva en todos y ningun cruce entre la familia del servidor y la del
//! cliente salvo `ramal-ai`. `Cargo.lock` incluye las dev-dependencies, asi que
//! las reglas cubren tambien los tests.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "tests fail loudly on purpose; a panic here is the assertion"
)]
#![allow(
    unused_crate_dependencies,
    reason = "the lint runs per target and this test only reads Cargo.lock"
)]

mod support;

use std::collections::BTreeSet;

use support::lockfile::{Graph, graph, parse, transitive_closure};

/// Crates del servidor.
const SERVER_CRATES: &[&str] = &[
    "ramal-api-domain",
    "ramal-api-application",
    "ramal-api-infrastructure",
    "ramal-api-http",
    "ramal-api",
];

/// Biblioteca neutral: la unica arista permitida entre familias.
const NEUTRAL_CRATES: &[&str] = &["ramal-ai"];

/// Infraestructura que el nucleo no puede alcanzar ni de forma transitiva.
///
/// `serde` no esta y no es un olvido: `Cargo.lock` apunta las dependencias
/// opcionales activadas por cualquier crate del workspace (la feature `serde` de
/// `chrono`, por ejemplo), asi que en el nucleo `serde` se controla con la lista
/// blanca directa.
const CORE_FORBIDDEN: &[&str] = &[
    "tokio",
    "sqlx",
    "sqlx-core",
    "sqlx-postgres",
    "salvo",
    "salvo_core",
    "salvo-oapi",
    "reqwest",
    "hyper",
    "figment",
    "argon2",
    "jsonwebtoken",
    "lettre",
    "aes-gcm",
];

/// Lo que puede declarar el dominio.
const DOMAIN_ALLOWED: &[&str] = &["chrono", "thiserror", "uuid"];

/// Lo que puede declarar la capa de aplicacion.
const APPLICATION_ALLOWED: &[&str] = &[
    "async-trait",
    "chrono",
    "ramal-api-domain",
    "thiserror",
    "uuid",
];

fn is_server_crate(name: &str) -> bool {
    name == "ramal-api" || name.starts_with("ramal-api-")
}

fn is_client_crate(name: &str) -> bool {
    name.starts_with("ramal-") && !is_server_crate(name) && !NEUTRAL_CRATES.contains(&name)
}

fn assert_never_reaches(graph: &Graph, owner: &str, forbidden: &[&str]) {
    let reachable = transitive_closure(graph, owner);
    for name in forbidden {
        assert!(
            !reachable.contains(*name),
            "`{owner}` reaches `{name}`. The hexagon leaks: move that code to an adapter."
        );
    }
}

fn assert_direct_dependencies_are_allowed(graph: &Graph, owner: &str, allowed: &[&str]) {
    let direct = graph.get(owner).expect("the crate should be in Cargo.lock");
    for dependency in direct {
        assert!(
            allowed.contains(&dependency.as_str()),
            "`{owner}` declares `{dependency}`, which is not on its allowlist {allowed:?}. \
             The hexagon leaks: move that code to an adapter, or justify it in \
             docs/architecture.md and here."
        );
    }
}

#[test]
fn the_lock_file_describes_every_server_crate() {
    let graph = graph();
    for name in SERVER_CRATES {
        assert!(
            graph.contains_key(*name),
            "{name} is missing from Cargo.lock; the parser is out of date"
        );
    }
}

#[test]
fn the_parser_sees_real_edges() {
    let graph = graph();
    let http = transitive_closure(&graph, "ramal-api-http");
    assert!(
        http.contains("salvo"),
        "the http adapter should reach salvo"
    );
    let infrastructure = transitive_closure(&graph, "ramal-api-infrastructure");
    assert!(
        infrastructure.contains("sqlx"),
        "the persistence adapter should reach sqlx"
    );
    let binary = transitive_closure(&graph, "ramal-api");
    assert!(
        binary.contains("ramal-api-domain"),
        "the binary should reach the domain"
    );
}

#[test]
fn the_domain_only_declares_pure_libraries() {
    assert_direct_dependencies_are_allowed(&graph(), "ramal-api-domain", DOMAIN_ALLOWED);
}

#[test]
fn the_application_only_declares_the_domain_and_pure_libraries() {
    assert_direct_dependencies_are_allowed(&graph(), "ramal-api-application", APPLICATION_ALLOWED);
}

#[test]
fn the_core_never_reaches_infrastructure() {
    let graph = graph();
    for core in ["ramal-api-domain", "ramal-api-application"] {
        assert_never_reaches(&graph, core, CORE_FORBIDDEN);
    }
}

#[test]
fn the_persistence_adapter_never_reaches_http() {
    assert_never_reaches(
        &graph(),
        "ramal-api-infrastructure",
        &["salvo", "salvo_core", "salvo-oapi", "ramal-api-http"],
    );
}

#[test]
fn the_http_adapter_never_reaches_sql() {
    assert_never_reaches(
        &graph(),
        "ramal-api-http",
        &[
            "sqlx",
            "sqlx-core",
            "sqlx-postgres",
            "ramal-api-infrastructure",
        ],
    );
}

#[test]
fn no_server_crate_reaches_the_client_family() {
    let graph = graph();
    for owner in SERVER_CRATES {
        let reachable = transitive_closure(&graph, owner);
        let leaked: Vec<&String> = reachable
            .iter()
            .filter(|name| is_client_crate(name))
            .collect();
        assert!(
            leaked.is_empty(),
            "`{owner}` reaches the client crates {leaked:?}. The hexagon leaks: only \
             `ramal-ai` may be shared, and the HTTP contract is modelled on each side."
        );
    }
}

#[test]
fn no_client_crate_reaches_the_server_family() {
    let graph = graph();
    for owner in graph.keys().filter(|name| is_client_crate(name)) {
        let reachable = transitive_closure(&graph, owner);
        let leaked: Vec<&String> = reachable
            .iter()
            .filter(|name| is_server_crate(name))
            .collect();
        assert!(
            leaked.is_empty(),
            "`{owner}` reaches the server crates {leaked:?}. The hexagon leaks: the client \
             models the HTTP contract with its own types."
        );
    }
}

#[test]
fn the_neutral_library_depends_on_no_ramal_crate() {
    let graph = graph();
    for neutral in NEUTRAL_CRATES
        .iter()
        .filter(|name| graph.contains_key(**name))
    {
        let reachable = transitive_closure(&graph, neutral);
        assert!(
            reachable.iter().all(|name| !name.starts_with("ramal-")),
            "`{neutral}` must not depend on any ramal crate: it is shared by both hexagons"
        );
    }
}

#[test]
fn core_clippy_files_repeat_the_root_thresholds_and_forbid_io() {
    support::clippy::assert_core_clippy_files();
}

#[test]
fn a_commented_or_misplaced_clippy_entry_forbids_nothing() {
    let text = r#"
disallowed-types = [
  { path = "std::env::var", reason = "x" },
]
disallowed-methods = [
  # { path = "std::env::var", reason = "x" },
  { path = "std::fs::read", reason = "x" }, # { path = "std::fs::write" }
]
"#;
    let methods = support::clippy::declared_paths(text, "disallowed-methods");
    assert_eq!(methods, BTreeSet::from(["std::fs::read".to_owned()]));
}

#[test]
fn the_parser_reads_names_versions_and_nested_edges() {
    let lock = r#"
version = 4

[[package]]
name = "a"
version = "0.1.0"
dependencies = [
 "b",
 "c 1.0.0",
]

[[package]]
name = "b"
version = "0.1.0"
dependencies = [
 "c 2.0.0",
]

[[package]]
name = "c"
version = "1.0.0"
"#;
    let graph = parse(lock);
    assert_eq!(graph["a"], BTreeSet::from(["b".to_owned(), "c".to_owned()]));
    assert!(graph["c"].is_empty());
    let closure = transitive_closure(&graph, "a");
    assert_eq!(closure, BTreeSet::from(["b".to_owned(), "c".to_owned()]));
}
