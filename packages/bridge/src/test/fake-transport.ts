import type { BridgeErrorKind } from '../bridge-error';
import type { ErrorKind, JsonValue, RpcNotification, RpcRequest, RpcResponse } from '../generated';
import type { NotificationListener, Transport, Unsubscribe } from '../transport';

type Responder = (request: RpcRequest) => Promise<RpcResponse | null>;

/** Transporte falso: guarda lo que recibe y responde lo que se le diga. */
export class FakeTransport implements Transport {
  readonly requests: RpcRequest[] = [];
  readonly notifications: RpcNotification[] = [];
  respond: Responder = (request) => Promise.resolve(success(request, {}));
  readonly #listeners = new Set<NotificationListener>();

  request(request: RpcRequest): Promise<RpcResponse | null> {
    this.requests.push(request);
    return this.respond(request);
  }

  notify(notification: RpcNotification): void {
    this.notifications.push(notification);
  }

  subscribe(listener: NotificationListener): Unsubscribe {
    this.#listeners.add(listener);
    return () => {
      this.#listeners.delete(listener);
    };
  }

  emit(notification: RpcNotification): void {
    for (const listener of this.#listeners) listener(notification);
  }
}

/** Respuesta correcta a una peticion. */
export function success(request: RpcRequest, result: JsonValue): RpcResponse {
  return { jsonrpc: '2.0', id: request.id ?? null, result };
}

/** Respuesta con error del motor. */
export function failure(
  request: RpcRequest,
  code: number,
  kind: ErrorKind | BridgeErrorKind | null,
): RpcResponse {
  const error =
    kind === null
      ? { code, message: 'engine failed' }
      : { code, message: 'engine failed', data: { kind: kind as ErrorKind } };
  return { jsonrpc: '2.0', id: request.id ?? null, error };
}

/** Promesa que no termina nunca: un motor que no contesta. */
export function never(): Promise<RpcResponse | null> {
  return new Promise<RpcResponse | null>(() => undefined);
}

/** Signal manual que cumple lo que el cliente usa de `AbortSignal`. */
export function manualSignal(): {
  signal: {
    aborted: boolean;
    reason?: unknown;
    addEventListener: (type: 'abort', listener: () => void) => void;
    removeEventListener: (type: 'abort', listener: () => void) => void;
  };
  abort: (reason?: unknown) => void;
  listenerCount: () => number;
} {
  const listeners = new Set<() => void>();
  const signal = {
    aborted: false,
    reason: undefined as unknown,
    addEventListener: (_type: 'abort', listener: () => void) => listeners.add(listener),
    removeEventListener: (_type: 'abort', listener: () => void) => listeners.delete(listener),
  };
  return {
    signal,
    abort: (reason?: unknown) => {
      signal.aborted = true;
      signal.reason = reason;
      for (const listener of [...listeners]) listener();
    },
    listenerCount: () => listeners.size,
  };
}
