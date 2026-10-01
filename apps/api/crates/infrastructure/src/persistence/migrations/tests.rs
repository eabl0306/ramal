use super::{MIGRATOR, latest_migration_version};

#[test]
fn the_migration_set_is_embedded_and_not_empty() {
    assert!(MIGRATOR.iter().len() > 0);
}

#[test]
fn the_latest_version_is_the_newest_file() {
    let newest = MIGRATOR.iter().map(|m| m.version).max();
    assert_eq!(Some(latest_migration_version()), newest);
    assert!(latest_migration_version() >= 20_261_001_000_000);
}

#[test]
fn migrations_are_forward_only() {
    assert!(
        MIGRATOR
            .iter()
            .all(|migration| !migration.migration_type.is_down_migration()),
        "apps/api/migrations only takes forward migrations (ADR 0004)"
    );
}
