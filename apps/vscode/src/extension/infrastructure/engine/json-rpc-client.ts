import type { Buffer } from 'node:buffer';

import type { RpcNotification, RpcResponse } from '@ramal/bridge';

import type { IdentifiedRpcRequest } from '../../../shared/messages';
import { EngineUnavailableError } from '../../application/engine-errors';
import type { EngineConnection, Unsubscribe } from '../../application/ports/engine-connection';
import type { Logger } from '../../application/ports/logger';

import { encodeFrame } from './frame-codec';
import { isRpcNotification, isRpcResponse } from './rpc-guards';

interface Pending {
  readonly resolve: (response: RpcResponse) => void;
  readonly reject: (error: Error) => void;
}

/**
 * Cliente JSON-RPC sobre los marcos del sidecar. Correlaciona cada respuesta con
 * su peticion por el id que trae la peticion (lo reparte `RequestIds`, nunca
 * este cliente) y reparte las notificaciones. Al cerrarse, todas las peticiones
 * pendientes fallan con `EngineUnavailableError`.
 */
export class JsonRpcClient implements EngineConnection {
  readonly #write: (frame: Buffer) => void;
  readonly #logger: Logger;
  readonly #pending = new Map<number, Pending>();
  readonly #listeners = new Set<(notification: RpcNotification) => void>();
  #closed: string | null = null;

  constructor(write: (frame: Buffer) => void, logger: Logger) {
    this.#write = write;
    this.#logger = logger;
  }

  request(request: IdentifiedRpcRequest): Promise<RpcResponse> {
    if (this.#closed !== null) {
      return Promise.reject(new EngineUnavailableError(`the engine is gone: ${this.#closed}`));
    }
    if (this.#pending.has(request.id)) {
      return Promise.reject(new Error(`request id ${String(request.id)} is already in flight`));
    }
    return new Promise<RpcResponse>((resolve, reject) => {
      this.#pending.set(request.id, { resolve, reject });
      try {
        this.#write(encodeFrame(JSON.stringify(request)));
      } catch (error) {
        this.#pending.delete(request.id);
        reject(new EngineUnavailableError('could not write to the engine', { cause: error }));
      }
    });
  }

  onNotification(listener: (notification: RpcNotification) => void): Unsubscribe {
    this.#listeners.add(listener);
    return () => {
      this.#listeners.delete(listener);
    };
  }

  /** Un cuerpo de marco que llega del motor. */
  receive(body: string): void {
    let message: unknown;
    try {
      message = JSON.parse(body);
    } catch {
      this.#logger.warn('the engine sent a frame that is not JSON; ignored');
      return;
    }
    if (isRpcResponse(message)) {
      this.#settle(message);
      return;
    }
    if (isRpcNotification(message)) {
      for (const listener of this.#listeners) listener(message);
      return;
    }
    this.#logger.warn('the engine sent a message that is not JSON-RPC; ignored');
  }

  /** El proceso termino: nada mas va a contestar. */
  close(reason: string): void {
    if (this.#closed !== null) return;
    this.#closed = reason;
    const pending = [...this.#pending.values()];
    this.#pending.clear();
    for (const entry of pending) {
      entry.reject(new EngineUnavailableError(`the engine stopped: ${reason}`));
    }
  }

  #settle(response: RpcResponse): void {
    const entry = response.id === null ? undefined : this.#pending.get(response.id);
    if (entry === undefined || response.id === null) {
      this.#logger.warn(`the engine answered an unknown request ${String(response.id)}`);
      return;
    }
    this.#pending.delete(response.id);
    entry.resolve(response);
  }
}
