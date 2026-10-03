//! Migraciones y sonda de salud contra un PostgreSQL de verdad.
//!
//! Se saltan sin `GIT_RUDDER_TEST_DATABASE_URL` y fallan sin ella si
//! `GIT_RUDDER_TEST_REQUIRE_DATABASE=1`. Para lanzarlos: `just api-test-db`.

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

use std::time::Duration;

use git_rudder_api_application::HealthProbe;
use git_rudder_api_infrastructure::{
    PostgresHealthProbe, connect_pool, latest_migration_version, run_migrations,
};
use sqlx::{AssertSqlSafe, Executor};

const PROBE_TIMEOUT: Duration = Duration::from_secs(5);

#[tokio::test]
async fn migrations_create_the_schema_metadata_row() {
    let Some(pool) = support::migrated_pool().await else {
        return;
    };

    let version: String = sqlx::query_scalar("SELECT schema_version FROM schema_metadata")
        .fetch_one(&pool)
        .await
        .expect("the metadata row should exist");

    assert_eq!(version, "0.1.0");
}

#[tokio::test]
async fn migrations_are_idempotent() {
    let Some(pool) = support::migrated_pool().await else {
        return;
    };

    run_migrations(&pool)
        .await
        .expect("running migrations twice should be a no-op");

    let rows: i64 = sqlx::query_scalar("SELECT count(*) FROM schema_metadata")
        .fetch_one(&pool)
        .await
        .expect("count should succeed");
    assert_eq!(rows, 1);
}

#[tokio::test]
async fn citext_compares_without_case() {
    let Some(pool) = support::migrated_pool().await else {
        return;
    };

    let equal: bool =
        sqlx::query_scalar("SELECT 'Marta@Example.com'::citext = 'marta@example.com'")
            .fetch_one(&pool)
            .await
            .expect("citext should be installed");
    assert!(equal);
}

#[tokio::test]
async fn the_probe_reports_a_migrated_database_as_healthy() {
    let Some(pool) = support::migrated_pool().await else {
        return;
    };

    let probe = PostgresHealthProbe::new(pool, PROBE_TIMEOUT, latest_migration_version());
    assert_eq!(probe.name().as_str(), "database");
    probe
        .check()
        .await
        .expect("a reachable, migrated postgres should pass the probe");
}

#[tokio::test]
async fn the_probe_rejects_a_schema_behind_the_build() {
    let Some(pool) = support::migrated_pool().await else {
        return;
    };

    let probe = PostgresHealthProbe::new(pool, PROBE_TIMEOUT, latest_migration_version() + 1);
    let error = probe
        .check()
        .await
        .expect_err("the build expects a newer schema");
    assert!(error.message().contains("this build expects"));
}

#[tokio::test]
async fn the_probe_rejects_a_database_that_was_never_migrated() {
    let Some(url) = support::database_url() else {
        return;
    };
    // Un esquema vacio y propio hace de base sin migrar: con el search_path
    // apuntando ahi, la tabla de migraciones de SQLx no existe.
    let schema = format!("probe_empty_{}", support::unique_suffix());
    let admin = connect_pool(&support::test_config(url.clone())).expect("pool should build");
    // El nombre lo genera el propio test, sin entrada externa.
    let create = AssertSqlSafe(format!("CREATE SCHEMA {schema}"));
    admin
        .execute(create)
        .await
        .expect("the empty schema should be created");

    let separator = if url.contains('?') { '&' } else { '?' };
    let scoped_url = format!("{url}{separator}options=-c%20search_path%3D{schema}");
    let scoped = connect_pool(&support::test_config(scoped_url)).expect("pool should build");
    let probe = PostgresHealthProbe::new(scoped, PROBE_TIMEOUT, latest_migration_version());
    let outcome = probe.check().await;

    let drop = AssertSqlSafe(format!("DROP SCHEMA {schema}"));
    admin
        .execute(drop)
        .await
        .expect("the empty schema should be dropped");
    let error = outcome.expect_err("an unmigrated database is not ready");
    assert_eq!(error.message(), "migrations have not been applied yet");
}
