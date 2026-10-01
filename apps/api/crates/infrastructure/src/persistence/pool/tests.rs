#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "tests fail loudly on purpose; a panic here is the assertion"
)]

use super::{PersistenceError, connect_pool};
use crate::config::DatabaseConfig;

#[tokio::test]
async fn a_lazy_pool_is_built_without_touching_the_network() {
    let pool = connect_pool(&DatabaseConfig::default()).expect("lazy pool should build");
    assert_eq!(pool.size(), 0);
}

#[tokio::test]
async fn an_invalid_connection_string_is_rejected_up_front() {
    let config = DatabaseConfig {
        url: "not-a-url".to_owned(),
        ..DatabaseConfig::default()
    };
    let error = connect_pool(&config).expect_err("the url is not valid");
    assert!(matches!(error, PersistenceError::Connect(_)));
    assert!(error.to_string().contains("postgres pool"));
}
