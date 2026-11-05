//! Notificaciones del motor a la interfaz: mensajes sin id que llegan sin que la
//! interfaz pregunte.
//!
//! Como los metodos, cada una se declara una vez en `define_notifications!` y de
//! ahi salen el nombre, el tipo de sus parametros y el `EngineNotifications` de
//! TypeScript.

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Una notificacion del motor con su tipo.
pub trait EngineNotification {
    /// Nombre `<area>/<verbo>`.
    const NAME: &'static str;
    /// Parametros.
    type Params: Serialize + DeserializeOwned + TS + 'static;
}

/// Recorre las notificaciones con sus tipos.
pub trait NotificationVisitor {
    /// Se llama una vez por notificacion, en orden de declaracion.
    fn visit<N: EngineNotification>(&mut self);
}

/// Parametros de `repo/changed`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export)]
pub struct RepoChangedParams {
    /// Repositorio que cambio.
    pub repo_id: String,
}

macro_rules! define_notifications {
    ($(
        $(#[$doc:meta])*
        $name:literal => $marker:ident($params:ty);
    )+) => {
        $(
            $(#[$doc])*
            #[derive(Debug, Clone, Copy, PartialEq, Eq)]
            pub struct $marker;

            impl EngineNotification for $marker {
                const NAME: &'static str = $name;
                type Params = $params;
            }
        )+

        /// Visita cada notificacion con sus tipos.
        pub fn visit_notifications(visitor: &mut impl NotificationVisitor) {
            $(visitor.visit::<$marker>();)+
        }
    };
}

define_notifications! {
    /// Cambio algo en el arbol de trabajo o en `.git` de un repositorio abierto
    /// (rafagas agrupadas). La interfaz vuelve a leer lo que ensena.
    "repo/changed" => RepoChangedNotification(RepoChangedParams);
}
