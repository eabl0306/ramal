//! Ejecuta un futuro sin runtime asincrono.

use std::future::Future;
use std::pin::pin;
use std::task::{Context, Poll, Waker};

/// Ejecuta un futuro hasta completarlo sin depender de tokio.
///
/// Los dobles de test terminan en el primer `poll`, asi que un bucle simple basta
/// y mantiene el nucleo libre de runtime. Si un futuro devolviera `Pending`, se
/// cede el hilo y se vuelve a intentar.
pub fn block_on<F: Future>(future: F) -> F::Output {
    let mut future = pin!(future);
    let mut context = Context::from_waker(Waker::noop());
    loop {
        match future.as_mut().poll(&mut context) {
            Poll::Ready(value) => return value,
            Poll::Pending => std::thread::yield_now(),
        }
    }
}
