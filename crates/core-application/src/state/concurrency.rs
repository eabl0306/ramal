//! Limite de tareas a la vez, sin runtime asincrono.
//!
//! El estado de fondo de los repositorios del inicio lanza un git por tarjeta, y
//! con cincuenta repositorios serian cincuenta procesos a la vez. Este limite
//! deja pasar `max` y el resto espera su turno. Es un semaforo minimo: el nucleo
//! no puede usar el de tokio.
//!
//! Al soltar un permiso se despierta a todos los que esperan y compiten otra vez.
//! Con los pocos que esperan aqui es mas sencillo que una cola justa, y no pierde
//! despertares: si el que iba a recibir el permiso se abandono, otro lo coge.

use std::future::Future;
use std::pin::Pin;
use std::sync::{Mutex, MutexGuard, PoisonError};
use std::task::{Context, Poll, Waker};

#[derive(Debug)]
struct LimitState {
    available: usize,
    waiters: Vec<Waker>,
}

/// Deja pasar como mucho `max` tareas a la vez.
#[derive(Debug)]
pub struct ConcurrencyLimit {
    max: usize,
    state: Mutex<LimitState>,
}

impl ConcurrencyLimit {
    /// Limite de `max` a la vez (al menos una).
    #[must_use]
    pub fn new(max: usize) -> Self {
        let max = max.max(1);
        Self {
            max,
            state: Mutex::new(LimitState {
                available: max,
                waiters: Vec::new(),
            }),
        }
    }

    /// Espera turno. El permiso se devuelve al soltarlo.
    #[must_use]
    pub const fn acquire(&self) -> Acquire<'_> {
        Acquire { limit: self }
    }

    /// Permisos en uso ahora.
    #[must_use]
    pub fn in_use(&self) -> usize {
        self.max - self.lock().available
    }

    fn lock(&self) -> MutexGuard<'_, LimitState> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn release(&self) {
        let waiters = {
            let mut state = self.lock();
            state.available += 1;
            std::mem::take(&mut state.waiters)
        };
        for waiter in waiters {
            waiter.wake();
        }
    }
}

/// Futuro de [`ConcurrencyLimit::acquire`].
#[derive(Debug)]
pub struct Acquire<'a> {
    limit: &'a ConcurrencyLimit,
}

impl<'a> Future for Acquire<'a> {
    type Output = Permit<'a>;

    fn poll(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        let limit = self.limit;
        let mut state = limit.lock();
        if state.available > 0 {
            state.available -= 1;
            return Poll::Ready(Permit { limit });
        }
        let waker = context.waker();
        if !state.waiters.iter().any(|known| known.will_wake(waker)) {
            state.waiters.push(waker.clone());
        }
        Poll::Pending
    }
}

/// Turno concedido; se devuelve al soltarlo.
#[derive(Debug)]
pub struct Permit<'a> {
    limit: &'a ConcurrencyLimit,
}

impl Drop for Permit<'_> {
    fn drop(&mut self) {
        self.limit.release();
    }
}

#[cfg(test)]
mod tests;
