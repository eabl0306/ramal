import { type BridgeClient, createBridgeClient } from '@ramal/bridge';

import type { InitMessage, Surface, WebviewReadyMessage } from '../../shared/messages';

import { readHostMessage } from './host-messages';
import type { MessageSource, VsCodeApi } from './vscode-api';
import { VsCodeTransport } from './vscode-transport';

/** Lo que un webview necesita del host: el cliente del motor y su `init`. */
export interface HostConnection {
  readonly client: BridgeClient;
  /** Se resuelve con el primer `init`: idioma, superficie y repositorio. */
  readonly init: Promise<InitMessage>;
  dispose(): void;
}

/**
 * Conecta un webview con el extension host: escucha sus mensajes, avisa con
 * `webview/ready` y espera el `init`. El webview nunca sabe de rutas: el host ya
 * abrio el repositorio y solo le pasa su `repoId`.
 */
export function connectToHost(
  api: VsCodeApi,
  source: MessageSource,
  surface: Surface,
): HostConnection {
  const transport = new VsCodeTransport(api);
  let resolveInit: (init: InitMessage) => void = () => undefined;
  const init = new Promise<InitMessage>((resolve) => {
    resolveInit = resolve;
  });
  const listener = (event: MessageEvent): void => {
    const message = readHostMessage(event.data);
    if (message === null) return;
    if (message.type === 'init') resolveInit(message);
    else transport.handle(message);
  };
  source.addEventListener('message', listener);
  const ready: WebviewReadyMessage = { type: 'webview/ready', surface };
  api.postMessage(ready);
  return {
    client: createBridgeClient(transport),
    init,
    dispose: () => {
      source.removeEventListener('message', listener);
    },
  };
}
