use super::{MAX_INCOMING_LENGTH, is_acceptable, log_request};

#[test]
fn accepts_a_reasonable_identifier() {
    assert!(is_acceptable("abc-123"));
    assert!(is_acceptable("01J9Z3Q4W5E6R7T8Y9U0I1O2P3"));
}

#[test]
fn rejects_empty_long_or_strange_identifiers() {
    assert!(!is_acceptable(""));
    assert!(!is_acceptable(&"a".repeat(MAX_INCOMING_LENGTH + 1)));
    assert!(!is_acceptable("two words"));
    assert!(!is_acceptable("tab\there"));
}

#[test]
fn logging_a_probe_or_a_request_never_panics() {
    log_request("id", "GET", "/health/live", 200);
    log_request("id", "GET", "/v1/workspaces", 404);
}
