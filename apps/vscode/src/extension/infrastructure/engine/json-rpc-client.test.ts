import type { Buffer } from 'node:buffer';

import type { RpcNotification } from '@ramal/bridge';
import { describe, expect, it } from 'vitest';

import { RecordingLogger } from '../../../test/host-fakes';
import { EngineUnavailableError } from '../../application/engine-errors';

import { FrameDecoder } from './frame-codec';
import { JsonRpcClient } from './json-rpc-client';
import { LineSplitter } from './line-splitter';

const REQUEST = { jsonrpc: '2.0', id: 3, method: 'engine/info', params: {} } as const;

function client(): { client: JsonRpcClient; sent: unknown[]; logger: RecordingLogger } {
  const decoder = new FrameDecoder();
  const sent: unknown[] = [];
  const logger = new RecordingLogger();
  const write = (frame: Buffer): void => {
    for (const body of decoder.push(frame)) sent.push(JSON.parse(body));
  };
  return { client: new JsonRpcClient(write, logger), sent, logger };
}

describe('cliente JSON-RPC del sidecar', () => {
  it('escribe la peticion en un marco y la resuelve con la respuesta de su id', async () => {
    const { client: rpc, sent } = client();
    const pending = rpc.request(REQUEST);
    expect(sent).toEqual([REQUEST]);
    rpc.receive('{"jsonrpc":"2.0","id":3,"result":{"protocolVersion":1}}');
    await expect(pending).resolves.toEqual({
      jsonrpc: '2.0',
      id: 3,
      result: { protocolVersion: 1 },
    });
  });

  it('un error del motor llega como respuesta, no como rechazo', async () => {
    const { client: rpc } = client();
    const pending = rpc.request(REQUEST);
    const error = { code: -32_000, message: 'no', data: { kind: 'git_failed' } };
    rpc.receive(JSON.stringify({ jsonrpc: '2.0', id: 3, error }));
    await expect(pending).resolves.toEqual({ jsonrpc: '2.0', id: 3, error });
  });

  it('reparte las notificaciones del motor', () => {
    const { client: rpc } = client();
    const received: RpcNotification[] = [];
    const unsubscribe = rpc.onNotification((notification) => received.push(notification));
    rpc.receive('{"jsonrpc":"2.0","method":"repo/changed","params":{"repoId":"r"}}');
    unsubscribe();
    rpc.receive('{"jsonrpc":"2.0","method":"repo/changed","params":{"repoId":"r"}}');
    expect(received).toEqual([{ jsonrpc: '2.0', method: 'repo/changed', params: { repoId: 'r' } }]);
  });

  it('ignora y registra lo que no es JSON, lo que no es JSON-RPC y los ids que no espera', () => {
    const { client: rpc, logger } = client();
    rpc.receive('not json');
    rpc.receive('{"hello":"world"}');
    rpc.receive('{"jsonrpc":"2.0","id":99,"result":{}}');
    rpc.receive('{"jsonrpc":"2.0","id":null,"error":{"code":-32700,"message":"parse"}}');
    rpc.receive('{"jsonrpc":"2.0","id":1,"result":{},"error":{"code":1,"message":"x"}}');
    expect(logger.messages('warn')).toEqual([
      'the engine sent a frame that is not JSON; ignored',
      'the engine sent a message that is not JSON-RPC; ignored',
      'the engine answered an unknown request 99',
      'the engine answered an unknown request null',
      'the engine sent a message that is not JSON-RPC; ignored',
    ]);
  });

  it('al cerrarse rechaza lo pendiente y lo que llegue despues con EngineUnavailableError', async () => {
    const { client: rpc } = client();
    const pending = rpc.request(REQUEST);
    rpc.close('exit code 101');
    rpc.close('again');
    await expect(pending).rejects.toThrow('the engine stopped: exit code 101');
    await expect(rpc.request({ ...REQUEST, id: 4 })).rejects.toBeInstanceOf(EngineUnavailableError);
  });

  it('no deja dos peticiones en vuelo con el mismo id', async () => {
    const { client: rpc } = client();
    void rpc.request(REQUEST);
    await expect(rpc.request(REQUEST)).rejects.toThrow('already in flight');
  });

  it('si no puede escribir, la peticion falla como motor no disponible', async () => {
    const rpc = new JsonRpcClient(() => {
      throw new Error('EPIPE');
    }, new RecordingLogger());
    await expect(rpc.request(REQUEST)).rejects.toBeInstanceOf(EngineUnavailableError);
  });
});

describe('lineas del log del motor', () => {
  it('parte en lineas aunque una llegue cortada y devuelve el resto al final', () => {
    const lines = new LineSplitter();
    expect(lines.push('INFO uno\r\nWARN do')).toEqual(['INFO uno']);
    expect(lines.push('s\nERROR tres')).toEqual(['WARN dos']);
    expect(lines.flush()).toBe('ERROR tres');
    expect(lines.flush()).toBeNull();
  });
});
