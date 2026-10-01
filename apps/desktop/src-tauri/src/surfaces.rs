//! Metodos del motor que cada ventana puede pedir por el comando `rpc`.
//!
//! El webview pinta contenido que no es de fiar (mensajes de commit, nombres de
//! rama, ficheros), asi que se trata como un cliente que puede estar comprometido:
//! una ventana solo llega a los metodos de su lista. Es la misma idea que
//! `application/surfaces.ts` del extension host de VS Code. Los nombres salen de
//! los marcadores de `define_methods!` (ramal-protocol), nunca de texto escrito a
//! mano, y los tests comprueban que ninguna lista contiene un metodo del host:
//! `repo/open` y `engine/info` no estan en ninguna, los llama solo el host.

use ramal_engine::mapping::error::{forbidden_method, method_not_found};
use ramal_protocol::methods::GraphWindowMethod;
use ramal_protocol::{MethodInfo, RpcError, RpcMethod as _};

/// Etiqueta de la ventana principal, la de `tauri.conf.json`.
pub const MAIN_WINDOW: &str = "main";

/// Notificaciones de control del protocolo que cualquier ventana puede mandar.
/// `$/cancel` la manda el cliente del bridge al cancelar una llamada; el motor del
/// primer corte todavia la ignora.
pub const CONTROL_NOTIFICATIONS: &[&str] = &["$/cancel"];

/// Una ventana y los metodos del motor que puede pedir.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Surface {
    /// Etiqueta de la ventana en Tauri.
    pub window: &'static str,
    /// Metodos permitidos, por su nombre `<area>/<verbo>`.
    pub methods: &'static [&'static str],
}

/// Las ventanas de la app. En el primer corte solo existe la principal, que solo
/// pinta el grafo.
pub const SURFACES: &[Surface] = &[Surface {
    window: MAIN_WINDOW,
    methods: &[GraphWindowMethod::NAME],
}];

/// Metodos permitidos a una ventana. Una ventana desconocida no tiene ninguno.
#[must_use]
pub fn methods_for(window: &str) -> &'static [&'static str] {
    SURFACES
        .iter()
        .find(|surface| surface.window == window)
        .map_or(&[], |surface| surface.methods)
}

/// Decide si `window` puede mandar `method`. Las peticiones con id solo pasan si el
/// metodo esta en la lista de la ventana; las notificaciones, si son de control.
///
/// # Errors
///
/// El mismo [`RpcError`] que daria el motor: un metodo del host o fuera de la lista
/// se responde como si no existiera, para no ensenar al webview que esta ahi.
pub fn authorize(window: &str, method: &str, is_notification: bool) -> Result<(), RpcError> {
    if is_notification {
        return if CONTROL_NOTIFICATIONS.contains(&method) {
            Ok(())
        } else {
            Err(method_not_found(method))
        };
    }
    if methods_for(window).contains(&method) {
        return Ok(());
    }
    match MethodInfo::find(method) {
        Some(_) => Err(forbidden_method(method)),
        None => Err(method_not_found(method)),
    }
}

#[cfg(test)]
mod tests;
