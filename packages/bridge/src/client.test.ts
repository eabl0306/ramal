import { afterEach, describe, expect, it, vi } from 'vitest';

import { BridgeError } from './bridge-error';
import { DEFAULT_TIMEOUT_MS, createBridgeClient, nextRequestId } from './client';
import type { GraphWindow } from './generated';
import { FakeTransport, failure, manualSignal, never, success } from './test/fake-transport';

const WINDOW: GraphWindow = {
  repoId: 'repo-1',
  offset: 0,
  rows: [],
  hasMore: false,
  laneCount: 0,
  snapshot: 'f'.repeat(32),
};

async function rejection(promise: Promise<unknown>): Promise<unknown> {
  return promise.then(
    () => {
      throw new Error('se esperaba un rechazo');
    },
    (error: unknown) => error,
  );
}

afterEach(() => {
  vi.useRealTimers();
});

describe('createBridgeClient', () => {
  it('manda un RpcRequest con id propio y devuelve el resultado tipado', async () => {
    const transport = new FakeTransport();
    transport.respond = (request) => Promise.resolve(success(request, WINDOW));
    const client = createBridgeClient(transport);

    const result = await client.call('graph/window', { repoId: 'repo-1', offset: 0, limit: 50 });

    expect(result).toEqual(WINDOW);
    expect(transport.requests).toEqual([
      {
        jsonrpc: '2.0',
        id: 1,
        method: 'graph/window',
        params: { repoId: 'repo-1', offset: 0, limit: 50 },
      },
    ]);
  });

  it('usa un id nuevo en cada llamada', async () => {
    const transport = new FakeTransport();
    const client = createBridgeClient(transport);

    await client.call('engine/info', {});
    await client.call('engine/info', {});

    expect(transport.requests.map((request) => request.id)).toEqual([1, 2]);
  });

  it('convierte un error del motor en BridgeError con su kind', async () => {
    const transport = new FakeTransport();
    transport.respond = (request) => Promise.resolve(failure(request, -32_000, 'repo_not_open'));
    const client = createBridgeClient(transport);

    const error = await rejection(
      client.call('graph/window', { repoId: 'x', offset: 0, limit: 1 }),
    );

    expect(error).toBeInstanceOf(BridgeError);
    expect(error).toMatchObject({ kind: 'repo_not_open', code: -32_000 });
  });

  it('acepta engine_unavailable del extension host aunque no sea del motor', async () => {
    const transport = new FakeTransport();
    transport.respond = (request) =>
      Promise.resolve(failure(request, -32_000, 'engine_unavailable'));

    const error = await rejection(createBridgeClient(transport).call('engine/info', {}));

    expect(error).toMatchObject({ kind: 'engine_unavailable' });
  });

  it('deja pasar un BridgeError que lanza el propio transporte', async () => {
    const transport = new FakeTransport();
    const own = new BridgeError('engine_unavailable', 'sidecar down');
    transport.respond = () => Promise.reject(own);

    const error = await rejection(createBridgeClient(transport).call('engine/info', {}));

    expect(error).toBe(own);
  });

  it('envuelve cualquier otro fallo del transporte como transport', async () => {
    const transport = new FakeTransport();
    const cause = new Error('ipc closed');
    transport.respond = () => Promise.reject(cause);

    const error = await rejection(createBridgeClient(transport).call('engine/info', {}));

    expect(error).toMatchObject({ kind: 'transport', message: 'ipc closed', cause });
  });

  it('trata un rechazo que no es un Error como transport', async () => {
    const transport = new FakeTransport();
    // eslint-disable-next-line @typescript-eslint/prefer-promise-reject-errors -- un transporte ajeno puede rechazar con cualquier valor, y eso es lo que se prueba
    transport.respond = () => Promise.reject('canal roto');

    const error = await rejection(createBridgeClient(transport).call('engine/info', {}));

    expect(error).toMatchObject({
      kind: 'transport',
      message: 'transport failed',
      cause: 'canal roto',
    });
  });

  it('rechaza una respuesta nula o con otro id', async () => {
    const empty = new FakeTransport();
    empty.respond = () => Promise.resolve(null);
    const mismatched = new FakeTransport();
    mismatched.respond = (request) =>
      Promise.resolve(success({ ...request, id: (request.id ?? 0) + 7 }, {}));

    expect(await rejection(createBridgeClient(empty).call('engine/info', {}))).toMatchObject({
      kind: 'transport',
    });
    expect(await rejection(createBridgeClient(mismatched).call('engine/info', {}))).toMatchObject({
      kind: 'transport',
    });
  });

  it('reenvia las notificaciones del transporte hasta que se cancela la suscripcion', () => {
    const transport = new FakeTransport();
    const client = createBridgeClient(transport);
    const seen: string[] = [];

    const unsubscribe = client.onNotification((notification) => seen.push(notification.method));
    transport.emit({ jsonrpc: '2.0', method: 'repo/changed', params: { repoId: 'repo-1' } });
    unsubscribe();
    transport.emit({ jsonrpc: '2.0', method: 'repo/changed', params: {} });

    expect(seen).toEqual(['repo/changed']);
  });
});

describe('timeouts', () => {
  it('rechaza con timeout a los 30 segundos y manda $/cancel', async () => {
    vi.useFakeTimers();
    const transport = new FakeTransport();
    transport.respond = never;
    const pending = rejection(createBridgeClient(transport).call('engine/info', {}));

    await vi.advanceTimersByTimeAsync(DEFAULT_TIMEOUT_MS);

    expect(await pending).toMatchObject({ kind: 'timeout' });
    expect(transport.notifications).toEqual([
      { jsonrpc: '2.0', method: '$/cancel', params: { id: 1 } },
    ]);
  });

  it('respeta un timeout por llamada y uno por defecto del cliente', async () => {
    vi.useFakeTimers();
    const transport = new FakeTransport();
    transport.respond = never;
    const client = createBridgeClient(transport, { defaultTimeoutMs: 100 });
    const byDefault = rejection(client.call('engine/info', {}));
    const custom = rejection(client.call('engine/info', {}, { timeoutMs: 1_000 }));

    await vi.advanceTimersByTimeAsync(100);
    expect(await byDefault).toMatchObject({ kind: 'timeout' });
    await vi.advanceTimersByTimeAsync(900);
    expect(await custom).toMatchObject({ kind: 'timeout' });
  });

  it('sin timeout espera lo que haga falta y no deja temporizadores', async () => {
    vi.useFakeTimers();
    const transport = new FakeTransport();
    let answer: (() => void) | undefined;
    transport.respond = (request) =>
      new Promise((resolve) => {
        answer = () => {
          resolve(success(request, { ok: true }));
        };
      });
    const pending = createBridgeClient(transport).call('engine/info', {}, { timeoutMs: null });

    await vi.advanceTimersByTimeAsync(10 * DEFAULT_TIMEOUT_MS);
    expect(vi.getTimerCount()).toBe(0);
    answer?.();

    await expect(pending).resolves.toEqual({ ok: true });
  });

  it('limpia el temporizador cuando la respuesta llega a tiempo', async () => {
    vi.useFakeTimers();
    const client = createBridgeClient(new FakeTransport());

    await client.call('engine/info', {});

    expect(vi.getTimerCount()).toBe(0);
  });
});

describe('cancelacion', () => {
  it('rechaza con el motivo del signal y manda $/cancel', async () => {
    const transport = new FakeTransport();
    transport.respond = never;
    const control = manualSignal();
    const reason = new Error('la pestana se cerro');
    const pending = rejection(
      createBridgeClient(transport).call('engine/info', {}, { signal: control.signal }),
    );

    control.abort(reason);

    expect(await pending).toBe(reason);
    expect(transport.notifications).toHaveLength(1);
    expect(control.listenerCount()).toBe(0);
  });

  it('un signal ya cancelado no llega a mandar la peticion', async () => {
    const transport = new FakeTransport();
    const control = manualSignal();
    control.abort('motivo sin Error');

    const error = await rejection(
      createBridgeClient(transport).call('engine/info', {}, { signal: control.signal }),
    );

    expect(error).toBeInstanceOf(Error);
    expect(error).toMatchObject({ cause: 'motivo sin Error' });
    expect(transport.requests).toHaveLength(0);
  });

  it('ignora un fallo al mandar $/cancel', async () => {
    const transport = new FakeTransport();
    transport.respond = never;
    transport.notify = () => {
      throw new Error('canal cerrado');
    };
    const control = manualSignal();
    const pending = rejection(
      createBridgeClient(transport).call('engine/info', {}, { signal: control.signal }),
    );

    control.abort(new Error('fuera'));

    expect(await pending).toMatchObject({ message: 'fuera' });
  });

  it('quita el listener del signal cuando la respuesta llega', async () => {
    const control = manualSignal();

    await createBridgeClient(new FakeTransport()).call(
      'engine/info',
      {},
      { signal: control.signal },
    );

    expect(control.listenerCount()).toBe(0);
  });
});

describe('nextRequestId', () => {
  it('cuenta desde 1 y vuelve a 1 al pasar de 2^31 - 1', () => {
    expect(nextRequestId(0)).toBe(1);
    expect(nextRequestId(41)).toBe(42);
    expect(nextRequestId(2_147_483_647)).toBe(1);
  });
});
