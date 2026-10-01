import { PROTOCOL_VERSION } from '@ramal/bridge';
import { describe, expect, it } from 'vitest';

import { FakeEngine, engineInfo, fail, ok } from '../../test/engine-fakes';

import { callEngine } from './call-engine';
import {
  EngineCallError,
  EngineUnavailableError,
  describeError,
  errorKey,
  errorKindOf,
} from './engine-errors';
import { createRequestIds } from './request-ids';
import { checkEngineProtocol } from './use-cases/check-engine-protocol';

describe('llamadas del host al motor', () => {
  it('manda la peticion con un id del contador y devuelve el resultado', async () => {
    const engine = new FakeEngine();
    engine.respond = (request) => Promise.resolve(ok(request, { repoId: 'repo-1' }));
    const ids = createRequestIds(41);
    const result = await callEngine(engine, ids, 'repo/open', { path: '/tmp/repo' });
    expect(result).toEqual({ repoId: 'repo-1' });
    expect(engine.requests).toEqual([
      { jsonrpc: '2.0', id: 42, method: 'repo/open', params: { path: '/tmp/repo' } },
    ]);
  });

  it('convierte un error del motor en EngineCallError con su tipo', async () => {
    const engine = new FakeEngine();
    engine.respond = (request) => Promise.resolve(fail(request, 'not_a_repository'));
    const error: unknown = await callEngine(engine, createRequestIds(), 'repo/open', {
      path: '/tmp',
    }).catch((caught: unknown) => caught);
    expect(error).toBeInstanceOf(EngineCallError);
    expect(errorKindOf(error)).toBe('not_a_repository');
    expect(errorKey(error)).toBe('errors.engine.not_a_repository');
  });
});

describe('tipos de error que ensena el host', () => {
  it('el motor ausente es engine_unavailable', () => {
    const error = new EngineUnavailableError('gone');
    expect(errorKindOf(error)).toBe('engine_unavailable');
    expect(errorKey(error)).toBe('errors.engine.engine_unavailable');
  });

  it('un error sin tipo o desconocido es interno', () => {
    const untyped = new EngineCallError({ code: -32_603, message: 'boom' });
    expect(errorKindOf(untyped)).toBe('internal');
    expect(errorKindOf(new Error('x'))).toBe('internal');
    expect(errorKindOf('texto')).toBe('internal');
  });

  it('describe errores y valores para el log', () => {
    expect(describeError(new EngineUnavailableError('gone'))).toBe('EngineUnavailableError: gone');
    expect(describeError(42)).toBe('42');
  });
});

describe('comprobacion del protocolo al arrancar', () => {
  it('acepta un motor con la misma version', async () => {
    const check = await checkEngineProtocol(new FakeEngine(), createRequestIds(), PROTOCOL_VERSION);
    expect(check).toEqual({ outcome: 'compatible', info: engineInfo(PROTOCOL_VERSION) });
  });

  it('detecta un motor de otra version como fallo de empaquetado', async () => {
    const engine = new FakeEngine();
    engine.respond = (request) => Promise.resolve(ok(request, engineInfo(7)));
    const check = await checkEngineProtocol(engine, createRequestIds(), PROTOCOL_VERSION);
    expect(check).toEqual({
      outcome: 'mismatch',
      engineVersion: 7,
      expected: PROTOCOL_VERSION,
    });
  });

  it('devuelve el fallo si el motor no contesta', async () => {
    const engine = new FakeEngine();
    const gone = new EngineUnavailableError('gone');
    engine.respond = () => Promise.reject(gone);
    expect(await checkEngineProtocol(engine, createRequestIds())).toEqual({
      outcome: 'failed',
      error: gone,
    });
  });

  it('separa el error que contesta el motor (sin git): el proceso no esta roto', async () => {
    const engine = new FakeEngine();
    engine.respond = (request) => Promise.resolve(fail(request, 'git_not_found'));
    const check = await checkEngineProtocol(engine, createRequestIds(), 1);
    expect(check.outcome).toBe('engine-error');
    expect(errorKindOf(check.outcome === 'engine-error' ? check.error : null)).toBe(
      'git_not_found',
    );
  });
});
