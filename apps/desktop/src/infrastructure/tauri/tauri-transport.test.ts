import { type RpcNotification, type RpcResponse, createBridgeClient } from '@git-rudder/bridge';
import { describe, expect, it, vi } from 'vitest';

import type { Invoke } from './invoke';
import { RPC_COMMAND, SUBSCRIBE_COMMAND, createTauriTransport } from './tauri-transport';

const window = {
  repoId: 'repo-1',
  offset: 0,
  rows: [],
  hasMore: false,
  laneCount: 1,
};

/** Un `invoke` que contesta como el comando `rpc` del host. */
function rpcInvoke(answer: (request: { id?: number }) => RpcResponse | null): Invoke {
  return vi.fn((_command: string, args?: Record<string, unknown>) =>
    Promise.resolve(answer(args?.['request'] as { id?: number })),
  ) as unknown as Invoke;
}

describe('createTauriTransport', () => {
  it('manda el sobre al comando rpc y el cliente del bridge recibe el resultado', async () => {
    const invoke = rpcInvoke((request) => ({
      jsonrpc: '2.0',
      id: request.id ?? null,
      result: window,
    }));
    const client = createBridgeClient(createTauriTransport(invoke));

    const result = await client.call('graph/window', { repoId: 'repo-1', offset: 0, limit: 50 });

    expect(result).toEqual(window);
    expect(invoke).toHaveBeenCalledWith(RPC_COMMAND, {
      request: {
        jsonrpc: '2.0',
        id: 1,
        method: 'graph/window',
        params: { repoId: 'repo-1', offset: 0, limit: 50 },
      },
    });
  });

  it('un error del motor llega dentro de la respuesta y el cliente lo convierte', async () => {
    const invoke = rpcInvoke((request) => ({
      jsonrpc: '2.0',
      id: request.id ?? null,
      error: {
        code: -32_000,
        message: 'repository repo-9 is not open',
        data: { kind: 'repo_not_open' },
      },
    }));
    const client = createBridgeClient(createTauriTransport(invoke));

    await expect(
      client.call('graph/window', { repoId: 'repo-9', offset: 0, limit: 50 }),
    ).rejects.toMatchObject({ kind: 'repo_not_open' });
  });

  it('si invoke rechaza, el cliente ve un fallo de transporte', async () => {
    const invoke = vi.fn(() => Promise.reject(new Error('ipc closed'))) as unknown as Invoke;
    const client = createBridgeClient(createTauriTransport(invoke));

    await expect(
      client.call('graph/window', { repoId: 'repo-1', offset: 0, limit: 50 }),
    ).rejects.toMatchObject({ kind: 'transport' });
  });

  it('manda las notificaciones por rpc y se traga sus fallos', async () => {
    const invoke = vi.fn(() => Promise.reject(new Error('ipc closed'))) as unknown as Invoke;
    const transport = createTauriTransport(invoke);
    const notification = { jsonrpc: '2.0' as const, method: '$/cancel', params: { id: 3 } };

    await expect(transport.notify(notification)).resolves.toBeUndefined();
    expect(invoke).toHaveBeenCalledWith(RPC_COMMAND, { request: notification });
  });

  it('la primera suscripcion abre un canal con rpc_subscribe y reparte lo que llega', async () => {
    const invoke = vi.fn(() => Promise.resolve(null)) as unknown as Invoke;
    let deliver: ((notification: RpcNotification) => void) | undefined;
    const channel = { id: 7 };
    const openChannel = vi.fn((onMessage: (notification: RpcNotification) => void) => {
      deliver = onMessage;
      return channel;
    });
    const transport = createTauriTransport(invoke, openChannel);
    const first = vi.fn();
    const second = vi.fn();

    transport.subscribe(first);
    const unsubscribe = transport.subscribe(second);
    await Promise.resolve();
    const changed: RpcNotification = {
      jsonrpc: '2.0',
      method: 'repo/changed',
      params: { repoId: 'repo-1' },
    };
    deliver?.(changed);
    unsubscribe();
    deliver?.(changed);

    expect(openChannel).toHaveBeenCalledTimes(1);
    expect(invoke).toHaveBeenCalledWith(SUBSCRIBE_COMMAND, { channel });
    expect(first).toHaveBeenCalledTimes(2);
    expect(second).toHaveBeenCalledTimes(1);
  });

  it('si rpc_subscribe falla, la siguiente suscripcion lo vuelve a intentar', async () => {
    const invoke = vi
      .fn()
      .mockRejectedValueOnce(new Error('no permission'))
      .mockResolvedValue(null) as unknown as Invoke;
    const openChannel = vi.fn(() => ({}));
    const transport = createTauriTransport(invoke, openChannel);

    transport.subscribe(vi.fn());
    await new Promise((resolve) => setTimeout(resolve, 0));
    transport.subscribe(vi.fn());

    expect(openChannel).toHaveBeenCalledTimes(2);
  });
});
