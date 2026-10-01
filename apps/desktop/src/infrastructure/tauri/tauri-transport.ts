import type {
  RpcNotification,
  RpcRequest,
  RpcResponse,
  Transport,
  Unsubscribe,
} from '@ramal/bridge';

import { type Invoke, tauriIpc } from './invoke';

/** Comando de Tauri que lleva el sobre JSON-RPC al motor (src-tauri/src/commands.rs). */
export const RPC_COMMAND = 'rpc';

/**
 * El `Transport` de @ramal/bridge sobre el comando `rpc` del host. `invoke` solo
 * rechaza por fallos de transporte: un error del motor llega como respuesta con
 * `error` y el cliente del bridge lo convierte en `BridgeError`.
 *
 * Las notificaciones (`$/cancel`) viajan por el mismo comando sin id y no tienen
 * respuesta. Las del motor hacia la interfaz llegaran por un `Channel` registrado
 * con `rpc_subscribe` (planificado); el primer corte no emite ninguna, asi que
 * suscribirse no hace nada.
 */
export function createTauriTransport(invoke: Invoke = tauriIpc): Transport {
  return {
    request: (request: RpcRequest) => invoke<RpcResponse | null>(RPC_COMMAND, { request }),
    notify: async (notification: RpcNotification) => {
      try {
        await invoke<null>(RPC_COMMAND, { request: notification });
      } catch {
        // Una notificacion no espera respuesta: si no llega, no hay a quien avisar.
      }
    },
    subscribe: (): Unsubscribe => () => undefined,
  };
}
