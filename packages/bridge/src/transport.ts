import type { RpcNotification, RpcRequest, RpcResponse } from './generated';

/** Recibe una notificacion del motor (progreso, cambios del repositorio...). */
export type NotificationListener = (notification: RpcNotification) => void;

/** Deja de escuchar. */
export type Unsubscribe = () => void;

/**
 * Puerto de transporte: como llega una peticion al motor y como vuelve su
 * respuesta. El cliente no sabe si habla con Tauri o con el extension host de
 * VS Code; cada app aporta su transporte.
 *
 * `request` solo rechaza por fallos del transporte. Un error del motor llega como
 * una respuesta con `error`, y el cliente lo convierte en `BridgeError`. Un
 * transporte puede rechazar con un `BridgeError` propio (por ejemplo
 * `engine_unavailable` cuando el sidecar no esta) y el cliente lo deja pasar.
 */
export interface Transport {
  /** Manda una peticion con id y espera su respuesta (`null` si no hubo). */
  request(request: RpcRequest): Promise<RpcResponse | null>;
  /** Manda una notificacion sin esperar respuesta (`$/cancel`). */
  notify(notification: RpcNotification): void | Promise<void>;
  /** Se suscribe a las notificaciones del motor. */
  subscribe(listener: NotificationListener): Unsubscribe;
}
