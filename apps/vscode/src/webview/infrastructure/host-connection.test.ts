import { describe, expect, it } from 'vitest';

import type { RpcRequestMessage } from '../../shared/messages';

import { connectToHost } from './host-connection';
import type { MessageSource, VsCodeApi } from './vscode-api';

/** El host de mentira: una ventana donde llegan mensajes y una API que los manda. */
function fakeHost(): {
  readonly api: VsCodeApi & { readonly posted: unknown[] };
  readonly source: MessageSource & { send(data: unknown): void; readonly count: () => number };
} {
  const posted: unknown[] = [];
  const listeners = new Set<(event: MessageEvent) => void>();
  return {
    api: { posted, postMessage: (message) => posted.push(message) },
    source: {
      addEventListener: (_type, listener) => listeners.add(listener),
      removeEventListener: (_type, listener) => listeners.delete(listener),
      send: (data) => {
        for (const listener of listeners) listener(new MessageEvent('message', { data }));
      },
      count: () => listeners.size,
    },
  };
}

describe('conexion del webview con el host', () => {
  it('avisa con webview/ready y espera el init', async () => {
    const { api, source } = fakeHost();
    const connection = connectToHost(api, source, 'graph');
    expect(api.posted).toEqual([{ type: 'webview/ready', surface: 'graph' }]);
    const init = {
      type: 'init',
      locale: 'es',
      surface: 'graph',
      repoId: 'repo-1',
      protocolVersion: 1,
    };
    source.send({ type: 'ruido' });
    source.send(init);
    await expect(connection.init).resolves.toEqual(init);
  });

  it('el cliente del motor hace la ida y vuelta por postMessage', async () => {
    const { api, source } = fakeHost();
    const connection = connectToHost(api, source, 'graph');
    const pending = connection.client.call('graph/window', {
      repoId: 'repo-1',
      offset: 0,
      limit: 10,
    });
    const sent = api.posted[1] as RpcRequestMessage;
    expect(sent.request.method).toBe('graph/window');
    const window = { repoId: 'repo-1', offset: 0, rows: [], hasMore: false, laneCount: 1 };
    source.send({
      type: 'rpc/response',
      response: { jsonrpc: '2.0', id: sent.request.id, result: window },
    });
    await expect(pending).resolves.toEqual(window);
  });

  it('al liberarse deja de escuchar', () => {
    const { api, source } = fakeHost();
    const connection = connectToHost(api, source, 'graph');
    expect(source.count()).toBe(1);
    connection.dispose();
    expect(source.count()).toBe(0);
  });
});
