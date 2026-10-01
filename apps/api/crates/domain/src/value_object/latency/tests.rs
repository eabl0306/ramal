use std::time::Duration;

use super::Latency;

#[test]
fn converts_a_duration_to_whole_milliseconds() {
    let latency = Latency::from_duration(Duration::from_micros(1_500));
    assert_eq!(latency.as_millis(), 1);
}

#[test]
fn zero_is_the_neutral_value() {
    assert_eq!(Latency::ZERO.as_millis(), 0);
    assert_eq!(Latency::default(), Latency::ZERO);
}

#[test]
fn saturates_instead_of_truncating() {
    let latency = Latency::from_duration(Duration::MAX);
    assert_eq!(latency.as_millis(), u64::MAX);
}

#[test]
fn keeps_the_original_duration() {
    let latency = Latency::from(Duration::from_millis(42));
    assert_eq!(latency.as_duration(), Duration::from_millis(42));
    assert_eq!(latency, Latency::from_millis(42));
}
