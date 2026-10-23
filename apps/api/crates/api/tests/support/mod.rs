//! Ayudas de los tests de `git-rudder-api`: el grafo de `Cargo.lock`, los
//! `clippy.toml` del nucleo y el binario lanzado como proceso.

#![allow(
    dead_code,
    reason = "each integration test binary compiles this module and uses only the part it needs"
)]
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "tests fail loudly on purpose; a panic here is the assertion"
)]

pub mod clippy;
pub mod http_client;
pub mod lockfile;
pub mod process;

use std::path::PathBuf;

/// Raiz del repositorio, donde viven `Cargo.lock` y `clippy.toml`.
pub fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../..")
}

/// Carpeta de la API (`apps/api`).
pub fn api_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// URL de la base de pruebas, o `None` si estos tests se pueden saltar.
///
/// Misma regla que los tests de `git-rudder-api-infrastructure`: sin
/// `GIT_RUDDER_TEST_DATABASE_URL` se saltan, y con `GIT_RUDDER_TEST_REQUIRE_DATABASE=1`
/// que falte es un fallo, para que la CI no de un verde falso.
pub fn database_url() -> Option<String> {
    let url = std::env::var("GIT_RUDDER_TEST_DATABASE_URL")
        .ok()
        .filter(|url| !url.is_empty());
    let mandatory = std::env::var("GIT_RUDDER_TEST_REQUIRE_DATABASE")
        .is_ok_and(|value| !matches!(value.trim(), "" | "0" | "false" | "FALSE"));
    assert!(
        url.is_some() || !mandatory,
        "GIT_RUDDER_TEST_DATABASE_URL is empty but GIT_RUDDER_TEST_REQUIRE_DATABASE is set: \
         these tests must not pass without a real postgres"
    );
    url
}
