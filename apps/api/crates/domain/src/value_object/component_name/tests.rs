#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "tests fail loudly on purpose; a panic here is the assertion"
)]

use super::{ComponentName, ComponentNameError, MAX_LENGTH};

#[test]
fn normalizes_case_and_surrounding_spaces() {
    let name = ComponentName::parse("  DataBase  ").expect("name should be valid");
    assert_eq!(name.as_str(), "database");
    assert_eq!(name.to_string(), "database");
}

#[test]
fn rejects_blank_names() {
    assert_eq!(ComponentName::parse("   "), Err(ComponentNameError::Empty));
}

#[test]
fn rejects_names_over_the_limit() {
    let raw = "a".repeat(MAX_LENGTH + 1);
    assert_eq!(
        ComponentName::parse(&raw),
        Err(ComponentNameError::TooLong(MAX_LENGTH + 1))
    );
}

#[test]
fn rejects_unsupported_characters() {
    assert_eq!(
        ComponentName::parse("data base"),
        Err(ComponentNameError::UnsupportedCharacter(' '))
    );
}

#[test]
fn accepts_digits_dashes_and_underscores() {
    assert!(ComponentName::parse("cache-2_primary").is_ok());
}

#[test]
fn errors_render_a_readable_reason() {
    assert_eq!(
        ComponentNameError::Empty.to_string(),
        "component name must not be empty"
    );
    assert!(ComponentNameError::TooLong(70).to_string().contains("70"));
    assert!(
        ComponentNameError::UnsupportedCharacter('!')
            .to_string()
            .contains('!')
    );
}
