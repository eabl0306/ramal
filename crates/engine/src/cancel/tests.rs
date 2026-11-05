#![allow(
    clippy::unwrap_used,
    reason = "tests fail loudly on purpose; a panic here is a failed assertion"
)]

use std::future::{Future, pending, ready};
use std::pin::pin;
use std::task::{Context, Poll, Waker};

use git_rudder_protocol::RequestId;

use super::Cancellations;

fn id(value: u32) -> RequestId {
    RequestId::new(value).unwrap()
}

#[test]
fn a_finished_request_returns_its_value_and_leaves_the_registry() {
    let cancellations = Cancellations::default();
    let mut context = Context::from_waker(Waker::noop());

    let poll = pin!(cancellations.track(id(1), ready(7))).poll(&mut context);

    assert_eq!(poll, Poll::Ready(Some(7)));
    assert!(!cancellations.cancel(id(1)), "nothing left to cancel");
}

#[test]
fn a_cancelled_request_stops_with_none() {
    let cancellations = Cancellations::default();
    let mut context = Context::from_waker(Waker::noop());
    let mut tracked = pin!(cancellations.track(id(2), pending::<u8>()));
    assert!(tracked.as_mut().poll(&mut context).is_pending());

    assert!(cancellations.cancel(id(2)));

    assert_eq!(tracked.as_mut().poll(&mut context), Poll::Ready(None));
    assert!(!cancellations.cancel(id(3)), "an unknown id is ignored");
}

#[test]
fn a_reused_id_does_not_drop_the_newer_request() {
    let cancellations = Cancellations::default();
    let older = cancellations.track(id(4), pending::<u8>());
    let newer = cancellations.track(id(4), pending::<u8>());

    drop(older);

    let mut context = Context::from_waker(Waker::noop());
    let mut newer = pin!(newer);
    assert!(
        cancellations.cancel(id(4)),
        "the newer one is still registered"
    );
    assert_eq!(newer.as_mut().poll(&mut context), Poll::Ready(None));
}
