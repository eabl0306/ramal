use super::ProbeError;

#[test]
fn keeps_and_renders_the_failure_reason() {
    let error = ProbeError::new("connection refused");
    assert_eq!(error.message(), "connection refused");
    assert_eq!(error.to_string(), "connection refused");
}
