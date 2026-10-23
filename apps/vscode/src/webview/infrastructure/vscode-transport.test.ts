import { BridgeError, type RpcNotification } from '@git-rudder/bridge';
import { describe, expect, it } from 'vitest';

import { isHostFailure, readHostMessage } from './host-messages';
import type { VsCodeApi } from './vscode-api';
import { VsCodeTransport } from './vscode-transport';

const REQUEST = { jsonrpc: '2.0', id: 4, method: 'graph/window', params: { repoId: 'r' } } as const;

function fakeApi(): VsCodeApi & { readonly posted: unknown[] } {
  const posted: unknown[] = [];
  return { posted, postMessage: (message) => posted.push(message) };
}

describe('transporte de VS Code', () => {
  it('manda rpc/request por postMessage y resuelve con la respuesta de su id', async () => {
    const api = fakeApi();
    const transport = new VsCodeTransport(api);
    const pending = transport.request(REQUEST);
    expect(api.posted).toEqual([{ type: 'rpc/request', request: REQUEST }]);
    transport.handle({ type: 'rpc/response', response: { jsonrpc: '2.0', id: 5, result: 'otra' } });
    transport.handle({
      type: 'rpc/response',
      response: { jsonrpc: '2.0', id: 4, result: { rows: [] } },
    });
    await expect(pending).resolves.toEqual({ jsonrpc: '2.0', id: 4, result: { rows: [] } });
  });

  it('un motor ausente llega como BridgeError engine_unavailable', async () => {
    const transport = new VsCodeTransport(fakeApi());
    const pending = transport.request(REQUEST);
    transport.handle({
      type: 'rpc/response',
      response: {
        jsonrpc: '2.0',
        id: 4,
        error: { code: -32_000, message: 'engine down', data: { kind: 'engine_unavailable' } },
      },
    });
    const error: unknown = await pending.catch((caught: unknown) => caught);
    expect(error).toBeInstanceOf(BridgeError);
    expect((error as BridgeError).kind).toBe('engine_unavailable');
  });

  it('un error del motor se entrega como respuesta para que el cliente lo traduzca', async () => {
    const transport = new VsCodeTransport(fakeApi());
    const pending = transport.request(REQUEST);
    const response = {
      jsonrpc: '2.0',
      id: 4,
      error: { code: -32_000, message: 'no repo', data: { kind: 'repo_not_open' } },
    } as const;
    transport.handle({ type: 'rpc/response', response });
    await expect(pending).resolves.toEqual(response);
  });

  it('reparte las notificaciones a quien este suscrito', () => {
    const transport = new VsCodeTransport(fakeApi());
    const received: RpcNotification[] = [];
    const unsubscribe = transport.subscribe((notification) => received.push(notification));
    const notification: RpcNotification = { jsonrpc: '2.0', method: 'repo/changed', params: {} };
    transport.handle({ type: 'rpc/notification', notification });
    unsubscribe();
    transport.handle({ type: 'rpc/notification', notification });
    expect(received).toEqual([notification]);
  });

  it('rechaza una peticion sin id y no manda las notificaciones de control', async () => {
    const api = fakeApi();
    const transport = new VsCodeTransport(api);
    await expect(
      transport.request({ jsonrpc: '2.0', method: 'graph/window', params: {} }),
    ).rejects.toThrow('a webview request needs an id');
    transport.notify({ jsonrpc: '2.0', method: '$/cancel', params: { id: 1 } });
    transport.handle({ type: 'host/response', id: 1, result: null });
    transport.handle({
      type: 'rpc/response',
      response: { jsonrpc: '2.0', id: null, result: null },
    });
    expect(api.posted).toEqual([]);
  });
});

describe('lectura de los mensajes del host', () => {
  it('reconoce los mensajes del host con su forma minima', () => {
    const init = { type: 'init', locale: 'es', surface: 'graph', repoId: 'r', protocolVersion: 1 };
    expect(readHostMessage(init)).toEqual(init);
    const response = { type: 'rpc/response', response: { jsonrpc: '2.0', id: 1, result: null } };
    expect(readHostMessage(response)).toEqual(response);
    const notification = { type: 'rpc/notification', notification: { method: 'x' } };
    expect(readHostMessage(notification)).toEqual(notification);
    expect(readHostMessage({ type: 'host/response', id: 1, result: null })).not.toBeNull();
  });

  it('ignora lo que no es del host', () => {
    for (const data of [
      null,
      'init',
      { type: 'init', locale: 'fr', surface: 'graph', repoId: 'r', protocolVersion: 1 },
      { type: 'init', locale: 'es', surface: 'terminal', repoId: 'r', protocolVersion: 1 },
      { type: 'rpc/response' },
      { type: 'rpc/notification', notification: 'x' },
      { type: 'host/response', id: '1' },
      { type: 'webpackOk' },
    ]) {
      expect(readHostMessage(data)).toBeNull();
    }
  });

  it('distingue las respuestas que fabrica el host', () => {
    expect(isHostFailure({ error: { data: { kind: 'engine_unavailable' } } })).toBe(true);
    expect(isHostFailure({ error: { data: { kind: 'repo_not_open' } } })).toBe(false);
    expect(isHostFailure({ result: 1 })).toBe(false);
    expect(isHostFailure(null)).toBe(false);
  });
});
