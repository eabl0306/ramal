//! Ejecutor de futuros sin runtime asincrono.

use std::future::Future;
use std::pin::pin;
use std::task::{Context, Poll, Waker};

/// Ejecuta un futuro hasta el final sin depender de tokio.
///
/// Los dobles de test terminan en el primer `poll`, asi que basta un bucle con un
/// waker que no hace nada. Asi los tests del nucleo no necesitan un runtime y el
/// test de arquitectura puede prohibir tokio tambien en las dev-dependencies.
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

#[cfg(test)]
mod tests {
    use std::future::Future;
    use std::pin::Pin;
    use std::task::{Context, Poll};

    use super::block_on;

    /// Futuro que responde `Pending` una vez antes de terminar.
    struct YieldOnce {
        yielded: bool,
    }

    impl Future for YieldOnce {
        type Output = u8;

        fn poll(mut self: Pin<&mut Self>, _context: &mut Context<'_>) -> Poll<u8> {
            if self.yielded {
                Poll::Ready(7)
            } else {
                self.yielded = true;
                Poll::Pending
            }
        }
    }

    #[test]
    fn returns_the_value_of_a_ready_future() {
        assert_eq!(block_on(async { 3 }), 3);
    }

    #[test]
    fn polls_again_after_a_pending_answer() {
        assert_eq!(block_on(YieldOnce { yielded: false }), 7);
    }
}
