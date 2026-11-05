//! `$/cancel`: parar una peticion en curso.
//!
//! Cada peticion con id que entra por [`Engine::handle_with`] se registra con su
//! id. Una notificacion `$/cancel` con ese id la marca, y su futuro deja de
//! sondearse y se suelta: el proceso de git que estuviera esperando muere con el
//! (`kill_on_drop`). La respuesta es un error `cancelled` con el codigo de LSP.
//!
//! No hace falta runtime: el futuro envuelto mira la marca en cada sondeo y la
//! cancelacion despierta al ultimo `Waker` que vio.
//!
//! [`Engine::handle_with`]: crate::Engine::handle_with

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::task::{Context, Poll, Waker};

use git_rudder_protocol::RequestId;

#[derive(Debug, Default)]
struct CancelState {
    cancelled: AtomicBool,
    waker: Mutex<Option<Waker>>,
}

impl CancelState {
    fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
        let waker = self
            .waker
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .take();
        if let Some(waker) = waker {
            waker.wake();
        }
    }
}

/// Peticiones en curso que se pueden cancelar.
#[derive(Debug, Default)]
pub struct Cancellations {
    pending: Mutex<HashMap<RequestId, Arc<CancelState>>>,
}

impl Cancellations {
    /// Envuelve el futuro de la peticion `id`. Devuelve `None` si se cancelo
    /// antes de terminar.
    pub fn track<F: Future>(&self, id: RequestId, future: F) -> Tracked<'_, F> {
        let state = Arc::new(CancelState::default());
        self.lock().insert(id, Arc::clone(&state));
        Tracked {
            owner: self,
            id,
            state,
            future: Box::pin(future),
        }
    }

    /// Cancela la peticion `id` si esta en curso. `false` si no estaba (ya
    /// termino o nunca existio): no es un error, la respuesta ya salio.
    pub fn cancel(&self, id: RequestId) -> bool {
        let state = self.lock().get(&id).cloned();
        state.map(|state| state.cancel()).is_some()
    }

    fn lock(&self) -> MutexGuard<'_, HashMap<RequestId, Arc<CancelState>>> {
        self.pending.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// Futuro de una peticion que se puede cancelar. Al soltarlo sale del registro.
pub struct Tracked<'a, F: Future> {
    owner: &'a Cancellations,
    id: RequestId,
    state: Arc<CancelState>,
    future: Pin<Box<F>>,
}

impl<F: Future> Future for Tracked<'_, F> {
    type Output = Option<F::Output>;

    fn poll(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        if self.state.cancelled.load(Ordering::SeqCst) {
            return Poll::Ready(None);
        }
        *self
            .state
            .waker
            .lock()
            .unwrap_or_else(PoisonError::into_inner) = Some(context.waker().clone());
        // Una cancelacion que llegara justo entre la comprobacion y guardar el
        // waker se ve aqui.
        if self.state.cancelled.load(Ordering::SeqCst) {
            return Poll::Ready(None);
        }
        self.future.as_mut().poll(context).map(Some)
    }
}

impl<F: Future> Drop for Tracked<'_, F> {
    fn drop(&mut self) {
        let mut pending = self.owner.lock();
        if pending
            .get(&self.id)
            .is_some_and(|state| Arc::ptr_eq(state, &self.state))
        {
            pending.remove(&self.id);
        }
    }
}

#[cfg(test)]
mod tests;
