import type { RpcNotification, RpcResponse } from '@ramal/bridge';

import type { IdentifiedRpcRequest } from '../../../shared/messages';

/** Deja de escuchar. */
export type Unsubscribe = () => void;

/**
 * Conexion con el motor. La implementan el proceso del sidecar (un marco
 * `Content-Length` por mensaje) y el supervisor que lo reinicia.
 *
 * `request` manda la peticion con el id que trae: quien llama reparte los ids
 * (`RequestIds`) para que el host y los webviews nunca choquen. Un error del
 * motor llega como respuesta con `error`; solo se rechaza con
 * `EngineUnavailableError` cuando el motor no esta o se cae a mitad.
 */
export interface EngineConnection {
  request(request: IdentifiedRpcRequest): Promise<RpcResponse>;
  onNotification(listener: (notification: RpcNotification) => void): Unsubscribe;
}
