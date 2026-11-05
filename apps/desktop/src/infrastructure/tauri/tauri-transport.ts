import type {
  NotificationListener,
  RpcNotification,
  RpcRequest,
  RpcResponse,
  Transport,
  Unsubscribe,
} from '@git-rudder/bridge';
import { Channel } from '@tauri-apps/api/core';

import { type Invoke, tauriIpc } from './invoke';

/** Comando de Tauri que lleva el sobre JSON-RPC al motor (src-tauri/src/commands.rs). */
export const RPC_COMMAND = 'rpc';

/** Comando con el que el webview abre su canal de notificaciones del motor. */
export const SUBSCRIBE_COMMAND = 'rpc_subscribe';

/** Lo que hace falta para recibir: un canal nuevo por el que llega cada mensaje. */
export type OpenChannel = (onMessage: (notification: RpcNotification) => void) => unknown;

const tauriChannel: OpenChannel = (onMessage) => new Channel<RpcNotification>(onMessage);

/**
 * El `Transport` de @git-rudder/bridge sobre los comandos del host. `invoke` solo
 * rechaza por fallos de transporte: un error del motor llega como respuesta con
 * `error` y el cliente del bridge lo convierte en `BridgeError`.
 *
 * Las notificaciones hacia el motor (`$/cancel`) viajan por `rpc` sin id y no
 * tienen respuesta. Las del motor hacia la interfaz (`repo/changed`) llegan por
 * un `Channel` que se abre con `rpc_subscribe` la primera vez que alguien se
 * suscribe; despues se reparten a todos los suscritos.
 */
export function createTauriTransport(
  invoke: Invoke = tauriIpc,
  openChannel: OpenChannel = tauriChannel,
): Transport {
  const listeners = new Set<NotificationListener>();
  let subscribed = false;
  const deliver = (notification: RpcNotification): void => {
    for (const listener of listeners) listener(notification);
  };
  return {
    request: (request: RpcRequest) => invoke<RpcResponse | null>(RPC_COMMAND, { request }),
    notify: async (notification: RpcNotification) => {
      try {
        await invoke<null>(RPC_COMMAND, { request: notification });
      } catch {
        // Una notificacion no espera respuesta: si no llega, no hay a quien avisar.
      }
    },
    subscribe: (listener: NotificationListener): Unsubscribe => {
      listeners.add(listener);
      if (!subscribed) {
        subscribed = true;
        // Sin canal la interfaz sigue funcionando: refresca al volver el foco.
        invoke<null>(SUBSCRIBE_COMMAND, { channel: openChannel(deliver) }).catch(() => {
          subscribed = false;
        });
      }
      return () => {
        listeners.delete(listener);
      };
    },
  };
}
