#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "tests fail loudly on purpose; a panic here is the assertion"
)]

use super::{ServiceVersion, ServiceVersionError};

#[test]
fn parses_a_well_formed_version() {
    let version = ServiceVersion::parse(" 1.20.3 ").expect("version should be valid");
    assert_eq!(version, ServiceVersion::new(1, 20, 3));
    assert_eq!(version.major(), 1);
    assert_eq!(version.minor(), 20);
    assert_eq!(version.patch(), 3);
    assert_eq!(version.to_string(), "1.20.3");
}

#[test]
fn rejects_a_missing_segment() {
    assert_eq!(
        ServiceVersion::parse("1.2"),
        Err(ServiceVersionError::MalformedCore("1.2".to_owned()))
    );
}

#[test]
fn rejects_extra_segments() {
    assert_eq!(
        ServiceVersion::parse("1.2.3.4"),
        Err(ServiceVersionError::MalformedCore("1.2.3.4".to_owned()))
    );
}

#[test]
fn rejects_prerelease_suffixes() {
    assert_eq!(
        ServiceVersion::parse("1.2.3-beta"),
        Err(ServiceVersionError::NonNumericSegment {
            segment: "3-beta".to_owned()
        })
    );
}

#[test]
fn orders_versions_by_significance() {
    assert!(ServiceVersion::new(1, 0, 0) < ServiceVersion::new(1, 0, 1));
    assert!(ServiceVersion::new(1, 9, 9) < ServiceVersion::new(2, 0, 0));
}

#[test]
fn errors_render_the_rejected_text() {
    let malformed = ServiceVersionError::MalformedCore("x".to_owned());
    assert!(malformed.to_string().contains("\"x\""));
    let segment = ServiceVersionError::NonNumericSegment {
        segment: "b".to_owned(),
    };
    assert!(segment.to_string().contains("\"b\""));
}
