import {
  type BridgeClient,
  type BridgeErrorKind,
  type ErrorKind,
  type GraphRow,
  type GraphWindow,
  type GraphWindowParams,
  type NotificationListener,
  type RpcNotification,
  type RpcRequest,
  type RpcResponse,
  type Transport,
  type Unsubscribe,
  createBridgeClient,
} from '@ramal/bridge';

/** Codigo JSON-RPC de cualquier error del motor. */
const ENGINE_ERROR_CODE = -32_000;

/**
 * Motor falso detras de un Transport de verdad: el cliente del bridge que usa la
 * vista es el real, asi los tests cubren tambien el sobre JSON-RPC, los ids y la
 * conversion de errores. Sirve `graph/window` recortando un historial fijo.
 */
export class FakeEngine implements Transport {
  /** Parametros de cada `graph/window` recibido, en orden. */
  readonly calls: GraphWindowParams[] = [];
  readonly notifications: RpcNotification[] = [];
  laneCount = 1;
  /** Mientras es verdadero, las respuestas esperan a `release()`. */
  holding = false;
  readonly #history: readonly GraphRow[];
  readonly #failures = new Map<number, ErrorKind | BridgeErrorKind>();
  readonly #held: (() => void)[] = [];

  constructor(history: readonly GraphRow[]) {
    this.#history = history;
  }

  /** La proxima peticion con este offset falla con el `kind` dado. */
  failAt(offset: number, kind: ErrorKind | BridgeErrorKind): void {
    this.#failures.set(offset, kind);
  }

  /** Suelta las respuestas retenidas. */
  release(): void {
    this.holding = false;
    for (const resolve of this.#held.splice(0)) resolve();
  }

  async request(request: RpcRequest): Promise<RpcResponse | null> {
    if (this.holding) await new Promise<void>((resolve) => this.#held.push(resolve));
    const params = request.params as GraphWindowParams;
    this.calls.push(params);
    const kind = this.#failures.get(params.offset);
    if (kind !== undefined) {
      this.#failures.delete(params.offset);
      return failure(request, kind);
    }
    return { jsonrpc: '2.0', id: request.id ?? null, result: this.window(params) };
  }

  notify(notification: RpcNotification): void {
    this.notifications.push(notification);
  }

  subscribe(_listener: NotificationListener): Unsubscribe {
    return () => undefined;
  }

  private window({ repoId, offset, limit }: GraphWindowParams): GraphWindow {
    const rows = this.#history.slice(offset, offset + limit);
    return {
      repoId,
      offset,
      rows,
      hasMore: offset + limit < this.#history.length,
      laneCount: this.laneCount,
      snapshot: 'snap-1',
    };
  }
}

function failure(request: RpcRequest, kind: ErrorKind | BridgeErrorKind): RpcResponse {
  return {
    jsonrpc: '2.0',
    id: request.id ?? null,
    // El extension host tambien manda kinds que no son del motor (engine_unavailable).
    error: { code: ENGINE_ERROR_CODE, message: 'engine failed', data: { kind: kind as ErrorKind } },
  };
}

/** Cliente del bridge real sobre el motor falso, con un timeout holgado para jsdom. */
export function fakeClient(engine: FakeEngine): BridgeClient {
  return createBridgeClient(engine, { defaultTimeoutMs: 60_000 });
}
