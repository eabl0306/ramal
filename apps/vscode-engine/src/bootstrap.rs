//! Composition root del sidecar: el `EngineBuilder` con la configuracion recibida.
//!
//! Aqui entraran los adaptadores que solo este host sabe construir, como el
//! `SecretStore` sobre peticiones inversas al extension host (planificado).

use ramal_engine::{Engine, EngineBuilder, EngineConfig};

use crate::args::Args;

/// Construye el motor con la configuracion del extension host.
#[must_use]
pub fn build_engine(args: &Args) -> Engine {
    EngineBuilder::new(EngineConfig {
        git_path: args.git_path.clone(),
    })
    .build()
}
