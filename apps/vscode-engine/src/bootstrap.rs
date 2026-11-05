//! Composition root del sidecar.
//!
//! El `EngineBuilder` con la configuracion recibida y lo que solo este host sabe
//! construir: las notificaciones como marcos por stdout y la carpeta de datos de
//! la extension para la confianza.
//!
//! Aqui entrara el `SecretStore` sobre peticiones inversas al extension host
//! (planificado).

use std::sync::Arc;

use git_rudder_engine::{Engine, EngineBuilder, EngineConfig, NotificationSink};

use crate::args::Args;

/// Fichero de la carpeta de datos donde se recuerda la confianza (ADR 0020).
pub const TRUST_FILE: &str = "trusted-repositories.txt";

/// Construye el motor con la configuracion del extension host.
#[must_use]
pub fn build_engine(args: &Args, sink: Arc<dyn NotificationSink>) -> Engine {
    EngineBuilder::new(EngineConfig {
        git_path: args.git_path.clone(),
        trust_file: args.data_dir.as_ref().map(|dir| dir.join(TRUST_FILE)),
    })
    .with_notification_sink(sink)
    .build()
}
