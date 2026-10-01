import { describe, expect, it } from 'vitest';

import { MAX_MESSAGE_BYTES, parseWebviewMessage } from './webview-message-schema';

const REQUEST = {
  jsonrpc: '2.0',
  id: 1,
  method: 'graph/window',
  params: { repoId: 'r', offset: 0, limit: 50 },
};

function rejected(raw: unknown): string {
  const parsed = parseWebviewMessage(raw);
  if (parsed.ok) throw new Error('the message should have been rejected');
  return parsed.reason;
}

describe('validacion de los mensajes de un webview', () => {
  it('acepta los tres mensajes de la union tal cual', () => {
    const messages = [
      { type: 'webview/ready', surface: 'graph' },
      { type: 'rpc/request', request: REQUEST },
      { type: 'host/request', id: 2, method: 'clipboard/writeText', params: { text: 'abc' } },
    ];
    for (const message of messages) {
      expect(parseWebviewMessage(message)).toEqual({ ok: true, message });
    }
  });

  it('rechaza campos de mas en cualquier nivel', () => {
    expect(rejected({ type: 'webview/ready', surface: 'graph', extra: 1 })).toContain(
      'invalid shape',
    );
    expect(rejected({ type: 'rpc/request', request: { ...REQUEST, path: '/etc' } })).toContain(
      'invalid shape',
    );
  });

  it('exige un id entero entre 0 y 2^31 - 1', () => {
    for (const id of [-1, 1.5, 2_147_483_648, '1', null]) {
      expect(rejected({ type: 'rpc/request', request: { ...REQUEST, id } })).toContain(
        'request.id',
      );
    }
    const max = { type: 'rpc/request', request: { ...REQUEST, id: 2_147_483_647 } };
    expect(parseWebviewMessage(max).ok).toBe(true);
  });

  it('exige que params sea un objeto JSON', () => {
    for (const params of [[], null, 'x', 3, { nested: () => 1 }]) {
      expect(rejected({ type: 'rpc/request', request: { ...REQUEST, params } })).toContain(
        'params',
      );
    }
  });

  it('rechaza metodos vacios, otra version de JSON-RPC y tipos desconocidos', () => {
    expect(rejected({ type: 'rpc/request', request: { ...REQUEST, method: '' } })).toContain(
      'method',
    );
    expect(rejected({ type: 'rpc/request', request: { ...REQUEST, jsonrpc: '1.0' } })).toContain(
      'jsonrpc',
    );
    expect(rejected({ type: 'rpc/notify' })).toContain('invalid shape');
    expect(rejected({ type: 'webview/ready', surface: 'terminal' })).toContain('invalid shape');
    expect(rejected('webview/ready')).toContain('invalid shape');
  });

  it('valida los parametros de cada metodo nativo y rechaza los desconocidos', () => {
    const tooLong = {
      type: 'host/request',
      id: 1,
      method: 'clipboard/writeText',
      params: { text: 'x'.repeat(100_001) },
    };
    expect(rejected(tooLong)).toContain('invalid shape');
    const unknown = {
      type: 'host/request',
      id: 1,
      method: 'shell/exec',
      params: { command: 'rm' },
    };
    expect(rejected(unknown)).toContain('invalid shape');
  });

  it('descarta lo que pasa de 1 MiB serializado', () => {
    const big = {
      type: 'rpc/request',
      request: { ...REQUEST, params: { blob: 'x'.repeat(MAX_MESSAGE_BYTES) } },
    };
    expect(rejected(big)).toMatch(/^the message has \d+ bytes$/u);
  });

  it('descarta lo que no se puede serializar', () => {
    const cyclic: Record<string, unknown> = { type: 'webview/ready' };
    cyclic['self'] = cyclic;
    expect(rejected(cyclic)).toBe('the message is not serializable');
    expect(rejected(undefined)).toBe('the message is not serializable');
    expect(rejected({ type: 'webview/ready', surface: 10n })).toBe(
      'the message is not serializable',
    );
  });
});
