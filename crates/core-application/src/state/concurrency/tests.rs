use std::future::Future;
use std::pin::pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::task::{Context, Poll, Wake, Waker};

use super::ConcurrencyLimit;

/// Cuenta cuantas veces se le despierta.
#[derive(Default)]
struct CountingWaker(AtomicUsize);

impl Wake for CountingWaker {
    fn wake(self: Arc<Self>) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}

#[test]
fn only_max_permits_are_handed_out_at_once() {
    let limit = ConcurrencyLimit::new(2);
    let counter = Arc::new(CountingWaker::default());
    let waker = Waker::from(Arc::clone(&counter));
    let mut context = Context::from_waker(&waker);

    let Poll::Ready(first) = pin!(limit.acquire()).poll(&mut context) else {
        unreachable!("the first permit is free")
    };
    let Poll::Ready(second) = pin!(limit.acquire()).poll(&mut context) else {
        unreachable!("the second permit is free")
    };
    assert_eq!(limit.in_use(), 2);

    let mut third = pin!(limit.acquire());
    assert!(third.as_mut().poll(&mut context).is_pending());
    assert!(third.as_mut().poll(&mut context).is_pending());

    drop(first);
    assert_eq!(
        counter.0.load(Ordering::SeqCst),
        1,
        "one waiter, woken once"
    );
    let Poll::Ready(third) = third.as_mut().poll(&mut context) else {
        unreachable!("a permit was released")
    };
    assert_eq!(limit.in_use(), 2);

    drop(second);
    drop(third);
    assert_eq!(limit.in_use(), 0);
}

#[test]
fn a_limit_of_zero_still_lets_one_through() {
    let limit = ConcurrencyLimit::new(0);
    let mut context = Context::from_waker(Waker::noop());

    let permit = pin!(limit.acquire()).poll(&mut context);

    assert!(permit.is_ready());
}
