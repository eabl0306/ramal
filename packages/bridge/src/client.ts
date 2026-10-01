import { BridgeError, fromRpcError, isBridgeError } from './bridge-error';
import type {
  EngineMethod,
  EngineMethods,
  RpcNotification,
  RpcRequest,
  RpcResponse,
} from './generated';
import type { NotificationListener, Transport, Unsubscribe } from './transport';

/** Timeout por defecto de una llamada. */
export const DEFAULT_TIMEOUT_MS = 30_000;

/** Mayor id de peticion: 2^31 - 1, un `number` seguro y lo que acepta el motor. */
const MAX_REQUEST_ID = 2_147_483_647;

/**
 * Lo que se usa de un `AbortSignal`. El paquete no carga los tipos del DOM ni de
 * Node, asi que se declara aqui; un `AbortSignal` de verdad encaja.
 */
export interface AbortSignalLike {
  readonly aborted: boolean;
  readonly reason?: unknown;
  addEventListener(type: 'abort', listener: () => void, options?: { once?: boolean }): void;
  removeEventListener(type: 'abort', listener: () => void): void;
}

/** Temporizadores. Por defecto, los del entorno. */
export interface Scheduler {
  setTimeout(callback: () => void, ms: number): unknown;
  clearTimeout(handle: unknown): void;
}

/** Opciones de una llamada. */
export interface CallOptions {
  /** Cancela la llamada: rechaza con `signal.reason` y manda `$/cancel`. */
  readonly signal?: AbortSignalLike;
  /** Tiempo maximo; `null` sin limite (operaciones largas con `$/progress`). */
  readonly timeoutMs?: number | null;
}

/** Opciones del cliente. */
export interface BridgeClientOptions {
  readonly defaultTimeoutMs?: number;
  readonly scheduler?: Scheduler;
}

/** Cliente tipado del motor. */
export interface BridgeClient {
  /** Llama a un metodo; rechaza con `BridgeError` si el motor o el transporte fallan. */
  call<M extends EngineMethod>(
    method: M,
    params: EngineMethods[M]['params'],
    options?: CallOptions,
  ): Promise<EngineMethods[M]['result']>;
  /** Se suscribe a las notificaciones del motor. */
  onNotification(listener: NotificationListener): Unsubscribe;
}

interface PendingCall {
  readonly transport: Transport;
  readonly scheduler: Scheduler;
  readonly request: RpcRequest;
  readonly timeoutMs: number | null;
  readonly signal: AbortSignalLike | undefined;
}

/** Crea el cliente sobre un transporte. */
export function createBridgeClient(
  transport: Transport,
  options: BridgeClientOptions = {},
): BridgeClient {
  const scheduler = options.scheduler ?? environmentScheduler();
  const defaultTimeout = options.defaultTimeoutMs ?? DEFAULT_TIMEOUT_MS;
  let lastId = 0;
  return {
    call<M extends EngineMethod>(
      method: M,
      params: EngineMethods[M]['params'],
      callOptions: CallOptions = {},
    ): Promise<EngineMethods[M]['result']> {
      lastId = nextRequestId(lastId);
      const pending: PendingCall = {
        transport,
        scheduler,
        request: { jsonrpc: '2.0', id: lastId, method, params },
        timeoutMs: callOptions.timeoutMs === undefined ? defaultTimeout : callOptions.timeoutMs,
        signal: callOptions.signal,
      };
      // El motor y los tipos salen del mismo `define_methods!`: el resultado de
      // `M` tiene la forma de `EngineMethods[M]['result']` por construccion.
      return callEngine(pending) as Promise<EngineMethods[M]['result']>;
    },
    onNotification: (listener) => transport.subscribe(listener),
  };
}

/** Siguiente id, volviendo a 1 al llegar al maximo. */
export function nextRequestId(previous: number): number {
  return previous >= MAX_REQUEST_ID ? 1 : previous + 1;
}

async function callEngine(call: PendingCall): Promise<unknown> {
  if (call.signal?.aborted === true) {
    throw abortReason(call.signal);
  }
  const response = await guardedRequest(call);
  return unwrap(response, call.request);
}

/** La peticion contra el timeout y la cancelacion: gana la primera que acaba. */
function guardedRequest(call: PendingCall): Promise<RpcResponse | null> {
  return new Promise<RpcResponse | null>((resolve, reject) => {
    const guards = startGuards(call, (error) => {
      sendCancel(call);
      reject(error);
    });
    call.transport.request(call.request).then(
      (response) => {
        guards.stop();
        resolve(response);
      },
      (error: unknown) => {
        guards.stop();
        reject(toTransportError(error));
      },
    );
  });
}

function startGuards(call: PendingCall, fail: (error: Error) => void): { stop: () => void } {
  let timer: unknown;
  const stop = (): void => {
    if (timer !== undefined) call.scheduler.clearTimeout(timer);
    call.signal?.removeEventListener('abort', onAbort);
  };
  function onAbort(): void {
    stop();
    fail(abortReason(call.signal));
  }
  if (call.timeoutMs !== null) {
    const { method } = call.request;
    const ms = call.timeoutMs;
    timer = call.scheduler.setTimeout(() => {
      stop();
      fail(new BridgeError('timeout', `${method} did not answer within ${String(ms)} ms`));
    }, ms);
  }
  call.signal?.addEventListener('abort', onAbort, { once: true });
  return { stop };
}

function unwrap(response: RpcResponse | null, request: RpcRequest): unknown {
  if (response === null) {
    throw new BridgeError('transport', `no response to ${request.method}`);
  }
  if (response.id !== request.id) {
    throw new BridgeError('transport', `response id does not match request ${request.method}`);
  }
  if ('error' in response) {
    throw fromRpcError(response.error);
  }
  return response.result;
}

/** Pide al motor que deje de trabajar en la peticion; si falla, da igual. */
function sendCancel(call: PendingCall): void {
  const notification: RpcNotification = {
    jsonrpc: '2.0',
    method: '$/cancel',
    params: { id: call.request.id ?? null },
  };
  void Promise.resolve()
    .then(() => call.transport.notify(notification))
    .catch(() => undefined);
}

function toTransportError(error: unknown): BridgeError {
  if (isBridgeError(error)) return error;
  const message = error instanceof Error ? error.message : 'transport failed';
  return new BridgeError('transport', message, { cause: error });
}

/** Lo que se rechaza al cancelar: el motivo del signal, como hace `fetch`. */
function abortReason(signal: AbortSignalLike | undefined): Error {
  const reason = signal?.reason;
  return reason instanceof Error ? reason : new Error('the call was aborted', { cause: reason });
}

/**
 * Los temporizadores del entorno (navegador, webview o Node). `library.json` no
 * trae los tipos del DOM ni de Node, asi que se leen de `globalThis` con el tipo
 * minimo, en cada llamada (los temporizadores falsos de los tests los sustituyen).
 */
function environmentScheduler(): Scheduler {
  const host = globalThis as unknown as Scheduler;
  return {
    setTimeout: (callback, ms) => host.setTimeout(callback, ms),
    clearTimeout: (handle) => {
      host.clearTimeout(handle);
    },
  };
}
