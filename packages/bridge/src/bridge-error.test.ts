import { describe, expect, it } from 'vitest';

import {
  BRIDGE_ERROR_KINDS,
  BridgeError,
  fromRpcError,
  isBridgeError,
  isBridgeErrorKind,
} from './bridge-error';
import { ENGINE_METHODS, HOST_METHODS, PROTOCOL_VERSION } from './generated';

describe('BridgeError', () => {
  it('guarda kind, codigo, datos y causa', () => {
    const cause = new Error('raiz');
    const error = new BridgeError('git_failed', 'git command failed', {
      code: -32_000,
      data: { kind: 'git_failed' },
      cause,
    });

    expect(error).toBeInstanceOf(Error);
    expect(error.name).toBe('BridgeError');
    expect(error).toMatchObject({ kind: 'git_failed', code: -32_000, cause });
    expect(error.data).toEqual({ kind: 'git_failed' });
    expect(isBridgeError(error)).toBe(true);
    expect(isBridgeError(cause)).toBe(false);
  });

  it('sin detalles deja codigo y datos vacios', () => {
    const error = new BridgeError('timeout', 'slow');

    expect(error.code).toBeUndefined();
    expect(error.data).toBeUndefined();
  });
});

describe('isBridgeErrorKind', () => {
  it('reconoce los kinds del motor y los tres del puente', () => {
    expect(BRIDGE_ERROR_KINDS).toHaveLength(10);
    for (const kind of ['repo_not_open', 'transport', 'timeout', 'engine_unavailable']) {
      expect(isBridgeErrorKind(kind)).toBe(true);
    }
  });

  it('rechaza cualquier otra cosa', () => {
    for (const value of ['cancelled', 'toString', 42, undefined, null]) {
      expect(isBridgeErrorKind(value)).toBe(false);
    }
  });
});

describe('fromRpcError', () => {
  it('usa el kind del motor', () => {
    const error = fromRpcError({
      code: -32_000,
      message: 'repository repo-9 is not open',
      data: { kind: 'repo_not_open' },
    });

    expect(error).toMatchObject({
      kind: 'repo_not_open',
      message: 'repository repo-9 is not open',
    });
  });

  it('sin kind conocido lo deduce del codigo', () => {
    expect(fromRpcError({ code: -32_602, message: 'bad' }).kind).toBe('invalid_params');
    expect(fromRpcError({ code: -32_603, message: 'boom' }).kind).toBe('internal');
  });
});

describe('tipos generados', () => {
  it('exponen la lista de metodos, los del host y la version del protocolo', () => {
    expect(ENGINE_METHODS).toEqual(['engine/info', 'repo/open', 'graph/window']);
    expect(HOST_METHODS).toEqual(['repo/open']);
    expect(PROTOCOL_VERSION).toBe(1);
  });
});
