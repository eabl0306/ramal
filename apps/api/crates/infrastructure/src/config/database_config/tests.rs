use super::DatabaseConfig;

#[test]
fn debug_never_prints_the_connection_string() {
    let config = DatabaseConfig {
        url: "postgres://user:hunter2@db/x".to_owned(),
        ..DatabaseConfig::default()
    };
    let rendered = format!("{config:?}");
    assert!(!rendered.contains("hunter2"));
    assert!(rendered.contains("<redacted>"));
    assert!(rendered.contains("max_connections: 5"));
}
