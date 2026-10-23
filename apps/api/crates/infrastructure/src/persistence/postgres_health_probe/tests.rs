#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "tests fail loudly on purpose; a panic here is the assertion"
)]

use std::time::Duration;

use git_rudder_api_application::HealthProbe;

use super::{PostgresHealthProbe, describe};
use crate::config::DatabaseConfig;
use crate::persistence::connect_pool;

fn unreachable_probe(millis: u64) -> PostgresHealthProbe {
    let config = DatabaseConfig {
        // Nadie escucha en el puerto 1: la conexion se rechaza al momento.
        url: "postgres://u:p@127.0.0.1:1/none".to_owned(),
        ..DatabaseConfig::default()
    };
    let pool = connect_pool(&config).expect("lazy pool should build");
    PostgresHealthProbe::new(pool, Duration::from_millis(millis), 1)
}

#[tokio::test]
async fn exposes_the_component_name_of_the_contract() {
    assert_eq!(unreachable_probe(50).name().as_str(), "database");
}

#[tokio::test]
async fn reports_an_error_when_postgres_is_unreachable() {
    let error = unreachable_probe(300)
        .check()
        .await
        .expect_err("port 1 should refuse");
    assert_ne!(error.message(), "");
}

#[tokio::test]
async fn a_schema_behind_the_build_is_not_ready() {
    let probe = unreachable_probe(50);
    let error = probe.compare(0).expect_err("0 is behind 1");
    assert_eq!(
        error.message(),
        "database schema is at migration 0, this build expects 1"
    );
    assert!(probe.compare(1).is_ok());
    assert!(
        probe.compare(2).is_ok(),
        "a newer schema is still compatible"
    );
}

#[test]
fn errors_that_are_not_from_the_database_keep_their_text() {
    let message = describe(&sqlx::Error::PoolTimedOut);
    assert!(message.contains("timed out"));
}
