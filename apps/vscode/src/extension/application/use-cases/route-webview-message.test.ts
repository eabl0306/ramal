import type { RpcResponse } from '@git-rudder/bridge';
import { describe, expect, it } from 'vitest';

import type { IdentifiedRpcRequest } from '../../../shared/messages';
import { FakeEngine, ok } from '../../../test/engine-fakes';
import { RecordingLogger, fakeChannel, flush } from '../../../test/host-fakes';
import { EngineCallError, EngineUnavailableError } from '../engine-errors';
import type { HostRequests } from '../ports/host-requests';
import { createRequestIds } from '../request-ids';

import {
  type RoutedRequest,
  type WebviewRouter,
  createWebviewRouter,
} from './route-webview-message';

function windowRequest(id: number): IdentifiedRpcRequest {
  return {
    jsonrpc: '2.0',
    id,
    method: 'graph/window',
    params: { repoId: 'repo-1', offset: 0, limit: 5 },
  };
}

interface Setup {
  readonly router: WebviewRouter;
  readonly engine: FakeEngine;
  readonly logger: RecordingLogger;
  readonly copied: string[];
  readonly routed: RoutedRequest[];
}

function setup(hostRequests?: HostRequests): Setup {
  const engine = new FakeEngine();
  engine.respond = (request) => Promise.resolve(ok(request, { echo: request.id }));
  const logger = new RecordingLogger();
  const copied: string[] = [];
  const router = createWebviewRouter({
    engine,
    ids: createRequestIds(1000),
    hostRequests: hostRequests ?? {
      handle: (_method, params) => {
        copied.push(params.text);
        return Promise.resolve(null);
      },
    },
    logger,
    locale: 'es',
    protocolVersion: 1,
  });
  const routed: RoutedRequest[] = [];
  router.onRouted((entry) => routed.push(entry));
  return { router, engine, logger, copied, routed };
}

describe('router de los mensajes de los webviews', () => {
  it('contesta webview/ready con init: idioma, superficie, repositorio y protocolo', async () => {
    const { router } = setup();
    const panel = fakeChannel('panel-1', 'graph', 'repo-7');
    router.attach(panel);
    await router.receive('panel-1', { type: 'webview/ready', surface: 'graph' });
    expect(panel.posted).toEqual([
      { type: 'init', locale: 'es', surface: 'graph', repoId: 'repo-7', protocolVersion: 1 },
    ]);
  });

  it('descarta un webview/ready que dice ser de otra superficie', async () => {
    const { router, logger } = setup();
    const panel = fakeChannel('panel-1', 'graph');
    router.attach(panel);
    await router.receive('panel-1', { type: 'webview/ready', surface: 'settings' });
    expect(panel.posted).toEqual([]);
    expect(logger.messages('warn')[0]).toContain('claims surface settings');
  });

  it('cambia el id hacia el motor y restaura el del webview en la respuesta', async () => {
    const { router, engine } = setup();
    const panel = fakeChannel('panel-1');
    router.attach(panel);
    await router.receive('panel-1', { type: 'rpc/request', request: windowRequest(7) });
    expect(engine.requests.map((request) => request.id)).toEqual([1001]);
    expect(panel.posted).toEqual([
      { type: 'rpc/response', response: { jsonrpc: '2.0', id: 7, result: { echo: 1001 } } },
    ]);
  });

  it('dos webviews con el mismo id no chocan: cada uno recibe la suya', async () => {
    const { router, engine } = setup();
    const first = fakeChannel('panel-1');
    const second = fakeChannel('panel-2');
    router.attach(first);
    router.attach(second);
    await Promise.all([
      router.receive('panel-1', { type: 'rpc/request', request: windowRequest(1) }),
      router.receive('panel-2', { type: 'rpc/request', request: windowRequest(1) }),
    ]);
    expect(new Set(engine.requests.map((request) => request.id)).size).toBe(2);
    expect(first.posted).toEqual([
      { type: 'rpc/response', response: { jsonrpc: '2.0', id: 1, result: { echo: 1001 } } },
    ]);
    expect(second.posted).toEqual([
      { type: 'rpc/response', response: { jsonrpc: '2.0', id: 1, result: { echo: 1002 } } },
    ]);
  });

  it('rechaza sin llegar al motor un metodo que la superficie no puede pedir', async () => {
    const { router, engine, routed } = setup();
    const panel = fakeChannel('panel-1');
    router.attach(panel);
    const request: IdentifiedRpcRequest = {
      jsonrpc: '2.0',
      id: 3,
      method: 'repo/open',
      params: { path: '/' },
    };
    await router.receive('panel-1', { type: 'rpc/request', request });
    expect(engine.requests).toEqual([]);
    expect(panel.posted).toEqual([
      {
        type: 'rpc/response',
        response: {
          jsonrpc: '2.0',
          id: 3,
          error: { code: -32_601, message: 'repo/open is not allowed on graph' },
        },
      },
    ]);
    expect(routed).toEqual([{ panelId: 'panel-1', method: 'repo/open', outcome: 'rejected' }]);
  });

  it('rechaza sin llegar al motor una peticion con el repoId de otro panel', async () => {
    const { router, engine, routed } = setup();
    const panel = fakeChannel('panel-1', 'graph', 'repo-2');
    router.attach(panel);
    await router.receive('panel-1', { type: 'rpc/request', request: windowRequest(6) });
    expect(engine.requests).toEqual([]);
    expect(panel.posted).toEqual([
      {
        type: 'rpc/response',
        response: {
          jsonrpc: '2.0',
          id: 6,
          error: {
            code: -32_000,
            message: 'graph/window asks for a repository that is not the one of panel-1',
            data: { kind: 'repo_not_open' },
          },
        },
      },
    ]);
    expect(routed).toEqual([{ panelId: 'panel-1', method: 'graph/window', outcome: 'rejected' }]);
  });

  it('si el motor no esta, el webview recibe engine_unavailable con su id', async () => {
    const { router, engine, routed } = setup();
    engine.respond = () => Promise.reject(new EngineUnavailableError('the engine is waiting'));
    const panel = fakeChannel('panel-1');
    router.attach(panel);
    await router.receive('panel-1', { type: 'rpc/request', request: windowRequest(9) });
    expect(panel.posted).toEqual([
      {
        type: 'rpc/response',
        response: {
          jsonrpc: '2.0',
          id: 9,
          error: {
            code: -32_000,
            message: 'EngineUnavailableError: the engine is waiting',
            data: { kind: 'engine_unavailable' },
          },
        },
      },
    ]);
    expect(routed[0]?.outcome).toBe('unavailable');
  });

  it('si el motor contesto con error al arrancar (sin git), el webview recibe ese tipo', async () => {
    const { router, engine, routed } = setup();
    const error = {
      code: -32_000,
      message: 'git executable not found',
      data: { kind: 'git_not_found' },
    } as const;
    engine.respond = () => Promise.reject(new EngineCallError(error));
    const panel = fakeChannel('panel-1');
    router.attach(panel);
    await router.receive('panel-1', { type: 'rpc/request', request: windowRequest(9) });
    expect(panel.posted).toEqual([
      { type: 'rpc/response', response: { jsonrpc: '2.0', id: 9, error } },
    ]);
    expect(routed[0]?.outcome).toBe('unavailable');
  });

  it('no entrega la respuesta a un panel que se cerro mientras tanto', async () => {
    const { router, engine } = setup();
    let answer: (response: RpcResponse) => void = () => undefined;
    engine.respond = () =>
      new Promise((resolve) => {
        answer = resolve;
      });
    const panel = fakeChannel('panel-1');
    router.attach(panel);
    const pending = router.receive('panel-1', { type: 'rpc/request', request: windowRequest(2) });
    await flush();
    router.detach('panel-1');
    answer(ok(windowRequest(1001), {}));
    await pending;
    expect(panel.posted).toEqual([]);
  });

  it('ignora mensajes de un panel que no conoce', async () => {
    const { router, engine, logger } = setup();
    await router.receive('panel-9', { type: 'rpc/request', request: windowRequest(1) });
    expect(engine.requests).toEqual([]);
    expect(logger.messages('warn')[0]).toContain('unknown panel panel-9');
  });

  it('resuelve las peticiones nativas en el host y contesta host/response', async () => {
    const { router, copied, engine } = setup();
    const panel = fakeChannel('panel-1');
    router.attach(panel);
    await router.receive('panel-1', {
      type: 'host/request',
      id: 4,
      method: 'clipboard/writeText',
      params: { text: 'abc1234' },
    });
    expect(copied).toEqual(['abc1234']);
    expect(engine.requests).toEqual([]);
    expect(panel.posted).toEqual([{ type: 'host/response', id: 4, result: null }]);
  });

  it('una peticion nativa que falla contesta con el error', async () => {
    const { router } = setup({ handle: () => Promise.reject(new Error('clipboard denied')) });
    const panel = fakeChannel('panel-1');
    router.attach(panel);
    await router.receive('panel-1', {
      type: 'host/request',
      id: 5,
      method: 'clipboard/writeText',
      params: { text: 'x' },
    });
    expect(panel.posted).toEqual([
      { type: 'host/response', id: 5, error: 'Error: clipboard denied' },
    ]);
  });

  it('reenvia las notificaciones del motor a todos los paneles hasta que se libera', () => {
    const { router, engine } = setup();
    const panel = fakeChannel('panel-1');
    router.attach(panel);
    const notification = { jsonrpc: '2.0', method: 'repo/changed', params: {} } as const;
    engine.emit(notification);
    router.dispose();
    engine.emit(notification);
    expect(panel.posted).toEqual([{ type: 'rpc/notification', notification }]);
    expect(engine.listenerCount).toBe(0);
  });

  it('avisa de cada peticion enrutada a quien escucha', async () => {
    const { router, routed } = setup();
    router.attach(fakeChannel('panel-1'));
    await router.receive('panel-1', { type: 'rpc/request', request: windowRequest(1) });
    expect(routed).toEqual([{ panelId: 'panel-1', method: 'graph/window', outcome: 'answered' }]);
  });
});
