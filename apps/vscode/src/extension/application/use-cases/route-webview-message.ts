import type { RequestId, RpcResponse } from '@ramal/bridge';
import type { Locale } from '@ramal/i18n';

import type {
  HostFailureResponse,
  HostRequestMessage,
  IdentifiedRpcRequest,
  WebviewLocale,
  WebviewToHostMessage,
} from '../../../shared/messages';
import { EngineCallError, describeError } from '../engine-errors';
import type { EngineConnection, Unsubscribe } from '../ports/engine-connection';
import type { HostRequests } from '../ports/host-requests';
import type { Logger } from '../ports/logger';
import type { PanelChannel } from '../ports/webview-surface';
import type { RequestIds } from '../request-ids';
import { isMethodAllowed } from '../surfaces';

/** Codigo JSON-RPC de metodo inexistente, el que recibe un metodo no permitido. */
const METHOD_NOT_FOUND = -32_601;
/** Codigo de error del motor; el host lo usa para `engine_unavailable`. */
const ENGINE_ERROR = -32_000;

/** Que paso con una peticion de un webview, para el log y los tests de integracion. */
export interface RoutedRequest {
  readonly panelId: string;
  readonly method: string;
  readonly outcome: 'answered' | 'rejected' | 'unavailable';
}

export interface WebviewRouterDeps {
  readonly engine: EngineConnection;
  readonly ids: RequestIds;
  readonly hostRequests: HostRequests;
  readonly logger: Logger;
  readonly locale: Locale;
  readonly protocolVersion: number;
}

/** El enrutado de los mensajes de todos los webviews abiertos. */
export interface WebviewRouter {
  attach(channel: PanelChannel): void;
  detach(panelId: string): void;
  /** Un mensaje ya validado en forma y tamano que llega del panel `panelId`. */
  receive(panelId: string, message: WebviewToHostMessage): Promise<void>;
  onRouted(listener: (routed: RoutedRequest) => void): Unsubscribe;
  dispose(): void;
}

interface InFlight {
  readonly panelId: string;
  readonly originalId: RequestId;
}

/**
 * La frontera entre los webviews y el motor (architecture.md 6.7, pasos 4 y 5).
 * La forma y el tamano ya los comprobo el esquema de zod; aqui se decide si el
 * metodo esta permitido para la superficie del panel y se cambia el id por uno
 * del host, guardando `(panel, id original)`. Asi un webview no puede chocar con
 * otro ni recibir la respuesta de otro.
 */
class Router implements WebviewRouter {
  readonly #deps: WebviewRouterDeps;
  readonly #panels = new Map<string, PanelChannel>();
  readonly #inFlight = new Map<number, InFlight>();
  readonly #listeners = new Set<(routed: RoutedRequest) => void>();
  readonly #unsubscribe: Unsubscribe;

  constructor(deps: WebviewRouterDeps) {
    this.#deps = deps;
    this.#unsubscribe = deps.engine.onNotification((notification) => {
      for (const panel of this.#panels.values()) {
        panel.post({ type: 'rpc/notification', notification });
      }
    });
  }

  attach(channel: PanelChannel): void {
    this.#panels.set(channel.id, channel);
  }

  detach(panelId: string): void {
    this.#panels.delete(panelId);
    for (const [hostId, entry] of this.#inFlight) {
      if (entry.panelId === panelId) this.#inFlight.delete(hostId);
    }
  }

  async receive(panelId: string, message: WebviewToHostMessage): Promise<void> {
    const panel = this.#panels.get(panelId);
    if (panel === undefined) {
      this.#deps.logger.warn(`message from an unknown panel ${panelId} dropped`);
      return;
    }
    switch (message.type) {
      case 'webview/ready':
        this.#ready(panel, message.surface);
        return;
      case 'rpc/request':
        await this.#forward(panel, message.request);
        return;
      case 'host/request':
        await this.#hostRequest(panel, message);
        return;
    }
  }

  onRouted(listener: (routed: RoutedRequest) => void): Unsubscribe {
    this.#listeners.add(listener);
    return () => {
      this.#listeners.delete(listener);
    };
  }

  dispose(): void {
    this.#unsubscribe();
    this.#panels.clear();
    this.#inFlight.clear();
  }

  #ready(panel: PanelChannel, surface: string): void {
    if (surface !== panel.surface) {
      this.#deps.logger.warn(`panel ${panel.id} claims surface ${surface}; dropped`);
      return;
    }
    // `Locale` de @ramal/i18n tiene que caber en el `WebviewLocale` de shared:
    // si uno de los dos cambia, esta asignacion deja de compilar.
    const locale: WebviewLocale = this.#deps.locale;
    panel.post({
      type: 'init',
      locale,
      surface: panel.surface,
      repoId: panel.repoId,
      protocolVersion: this.#deps.protocolVersion,
    });
  }

  async #forward(panel: PanelChannel, request: IdentifiedRpcRequest): Promise<void> {
    if (!isMethodAllowed(panel.surface, request.method)) {
      this.#deps.logger.warn(`${request.method} is not allowed on ${panel.surface}; rejected`);
      panel.post({ type: 'rpc/response', response: notAllowed(request, panel.surface) });
      this.#emit({ panelId: panel.id, method: request.method, outcome: 'rejected' });
      return;
    }
    const hostId = this.#deps.ids.next();
    this.#inFlight.set(hostId, { panelId: panel.id, originalId: request.id });
    let response: RpcResponse;
    try {
      response = await this.#deps.engine.request({ ...request, id: hostId });
    } catch (error) {
      this.#deps.logger.warn(`${request.method} failed: ${describeError(error)}`);
      this.#deliver(hostId, failure(hostId, error));
      this.#emit({ panelId: panel.id, method: request.method, outcome: 'unavailable' });
      return;
    }
    this.#deliver(hostId, response);
    this.#emit({ panelId: panel.id, method: request.method, outcome: 'answered' });
  }

  /** Devuelve la respuesta al panel que pidio, con su id original. */
  #deliver(hostId: number, response: RpcResponse | HostFailureResponse): void {
    const entry = this.#inFlight.get(hostId);
    this.#inFlight.delete(hostId);
    if (entry === undefined) return;
    const panel = this.#panels.get(entry.panelId);
    panel?.post({ type: 'rpc/response', response: { ...response, id: entry.originalId } });
  }

  async #hostRequest(panel: PanelChannel, message: HostRequestMessage): Promise<void> {
    try {
      const result = await this.#deps.hostRequests.handle(message.method, message.params);
      panel.post({ type: 'host/response', id: message.id, result });
    } catch (error) {
      this.#deps.logger.warn(`host request ${message.method} failed: ${describeError(error)}`);
      panel.post({ type: 'host/response', id: message.id, error: describeError(error) });
    }
  }

  #emit(routed: RoutedRequest): void {
    for (const listener of this.#listeners) listener(routed);
  }
}

function notAllowed(request: IdentifiedRpcRequest, surface: string): RpcResponse {
  return {
    jsonrpc: '2.0',
    id: request.id,
    error: { code: METHOD_NOT_FOUND, message: `${request.method} is not allowed on ${surface}` },
  };
}

/**
 * La respuesta cuando la peticion no llego al motor. Si el motor contesto con
 * error al arrancar (sin git), el webview recibe ese error con su tipo: con
 * `engine_unavailable` lo reintentaria en vano y ensenaria el texto equivocado.
 */
function failure(id: number, error: unknown): RpcResponse | HostFailureResponse {
  if (error instanceof EngineCallError) return { jsonrpc: '2.0', id, error: error.rpc };
  return unavailable(id, error);
}

function unavailable(id: number, error: unknown): HostFailureResponse {
  return {
    jsonrpc: '2.0',
    id,
    error: {
      code: ENGINE_ERROR,
      message: describeError(error),
      data: { kind: 'engine_unavailable' },
    },
  };
}

/** Crea el router de los webviews; se suscribe a las notificaciones del motor. */
export function createWebviewRouter(deps: WebviewRouterDeps): WebviewRouter {
  return new Router(deps);
}
