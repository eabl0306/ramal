/**
 * Mensajes entre un webview y el extension host (architecture.md 4.4). Los dos
 * lados los importan y nadie mas. Solo dependen de los tipos de @git-rudder/bridge:
 * los sobres `RpcRequest`, `RpcResponse` y `RpcNotification` salen de
 * `git-rudder-protocol`, asi que el webview y el motor hablan exactamente el mismo
 * JSON-RPC y el host solo lo reenvia.
 */
import type { RequestId, RpcNotification, RpcRequest, RpcResponse } from '@git-rudder/bridge';

/** Superficies de React: una por webview. El primer corte solo monta `graph`. */
export const SURFACES = ['graph', 'changes', 'merge', 'settings'] as const;

export type Surface = (typeof SURFACES)[number];

/**
 * Idioma de la interfaz. Repite el `Locale` de @git-rudder/i18n porque `shared` no
 * puede importarlo; el host comprueba en tipos que los dos coinciden.
 */
export type WebviewLocale = 'es' | 'en';

/**
 * Metodos nativos que un webview puede pedir al host, con sus parametros y su
 * resultado. Copiar es el unico del primer corte; los dialogos llegan con el
 * puerto RepoViewHost de @git-rudder/repo-view (planificado).
 */
export interface HostMethods {
  'clipboard/writeText': { params: { text: string }; result: null };
}

export type HostMethod = keyof HostMethods;

/** Una peticion del motor con su id: lo unico que un webview manda al motor. */
export type IdentifiedRpcRequest = RpcRequest & { id: RequestId };

// ---------------------------------------------------------------- webview -> host

/** El webview ya escucha: el host contesta con `init`. */
export interface WebviewReadyMessage {
  readonly type: 'webview/ready';
  readonly surface: Surface;
}

/** Una peticion al motor; el host la valida y le cambia el id. */
export interface RpcRequestMessage {
  readonly type: 'rpc/request';
  readonly request: IdentifiedRpcRequest;
}

/** Una peticion nativa que resuelve el host con la API de VS Code. */
export type HostRequestMessage = {
  [M in HostMethod]: {
    readonly type: 'host/request';
    readonly id: number;
    readonly method: M;
    readonly params: HostMethods[M]['params'];
  };
}[HostMethod];

export type WebviewToHostMessage = WebviewReadyMessage | RpcRequestMessage | HostRequestMessage;

// ---------------------------------------------------------------- host -> webview

/** Lo que el webview necesita para pintar: idioma, superficie y repositorio. */
export interface InitMessage {
  readonly type: 'init';
  readonly locale: WebviewLocale;
  readonly surface: Surface;
  readonly repoId: string;
  readonly protocolVersion: number;
}

/**
 * Tipos de error que genera el propio host y no el motor. El webview los
 * convierte en un `BridgeError` con ese `kind`.
 */
export type HostErrorKind = 'engine_unavailable';

/** Respuesta que el host fabrica cuando el motor no puede contestar. */
export interface HostFailureResponse {
  readonly jsonrpc: '2.0';
  readonly id: RequestId;
  readonly error: {
    readonly code: number;
    readonly message: string;
    readonly data: { readonly kind: HostErrorKind };
  };
}

export interface RpcResponseMessage {
  readonly type: 'rpc/response';
  readonly response: RpcResponse | HostFailureResponse;
}

export interface RpcNotificationMessage {
  readonly type: 'rpc/notification';
  readonly notification: RpcNotification;
}

/** Respuesta a un `host/request`: o resultado o error, nunca los dos. */
export type HostResponseMessage =
  | { readonly type: 'host/response'; readonly id: number; readonly result: unknown }
  | { readonly type: 'host/response'; readonly id: number; readonly error: string };

export type HostToWebviewMessage =
  InitMessage | RpcResponseMessage | RpcNotificationMessage | HostResponseMessage;
