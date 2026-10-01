import {
  BridgeError,
  type NotificationListener,
  type RpcNotification,
  type RpcRequest,
  type RpcResponse,
  type Transport,
  type Unsubscribe,
} from '@ramal/bridge';

import type { HostToWebviewMessage, RpcRequestMessage } from '../../shared/messages';

import { isHostFailure } from './host-messages';
import type { VsCodeApi } from './vscode-api';

interface Pending {
  readonly resolve: (response: RpcResponse) => void;
  readonly reject: (error: Error) => void;
}

/**
 * El `Transport` de @ramal/bridge en un webview de VS Code: cada peticion sale
 * por `postMessage` como `rpc/request` y vuelve como `rpc/response` con el
 * mismo id (el host lo cambia hacia el motor y lo restaura a la vuelta). Si el
 * motor no esta, el host contesta `engine_unavailable` y aqui se convierte en
 * un `BridgeError` con ese tipo.
 */
export class VsCodeTransport implements Transport {
  readonly #api: VsCodeApi;
  readonly #pending = new Map<number, Pending>();
  readonly #listeners = new Set<NotificationListener>();

  constructor(api: VsCodeApi) {
    this.#api = api;
  }

  request(request: RpcRequest): Promise<RpcResponse | null> {
    const { id } = request;
    if (id === undefined) {
      return Promise.reject(new BridgeError('transport', 'a webview request needs an id'));
    }
    return new Promise<RpcResponse>((resolve, reject) => {
      this.#pending.set(id, { resolve, reject });
      const message: RpcRequestMessage = { type: 'rpc/request', request: { ...request, id } };
      this.#api.postMessage(message);
    });
  }

  /**
   * `$/cancel` no sale del webview en el primer corte: el motor aun no cancela
   * y el host tendria que traducir el id (planificado con la cancelacion).
   */
  notify(_notification: RpcNotification): void {
    // Sin efecto a proposito; ver arriba.
  }

  subscribe(listener: NotificationListener): Unsubscribe {
    this.#listeners.add(listener);
    return () => {
      this.#listeners.delete(listener);
    };
  }

  /** Un mensaje del host ya leido. */
  handle(message: HostToWebviewMessage): void {
    if (message.type === 'rpc/notification') {
      for (const listener of this.#listeners) listener(message.notification);
      return;
    }
    if (message.type !== 'rpc/response') return;
    const { response } = message;
    const pending = response.id === null ? undefined : this.#pending.get(response.id);
    if (pending === undefined || response.id === null) return;
    this.#pending.delete(response.id);
    if (isHostFailure(response)) {
      pending.reject(new BridgeError('engine_unavailable', response.error.message));
      return;
    }
    pending.resolve(response);
  }
}
