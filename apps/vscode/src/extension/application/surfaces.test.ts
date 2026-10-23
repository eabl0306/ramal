import { ENGINE_METHODS, HOST_METHODS } from '@git-rudder/bridge';
import { describe, expect, it } from 'vitest';

import { SURFACES } from '../../shared/messages';

import { MAX_REQUEST_ID, createRequestIds } from './request-ids';
import { SURFACE_METHODS, isMethodAllowed } from './surfaces';

describe('lista de metodos por superficie', () => {
  it('el grafo solo puede pedir su ventana', () => {
    expect(isMethodAllowed('graph', 'graph/window')).toBe(true);
    expect(isMethodAllowed('graph', 'repo/open')).toBe(false);
    expect(isMethodAllowed('graph', 'engine/info')).toBe(false);
  });

  it('las superficies que aun no existen no pueden pedir nada', () => {
    for (const method of ENGINE_METHODS) {
      expect(isMethodAllowed('changes', method)).toBe(false);
      expect(isMethodAllowed('merge', method)).toBe(false);
      expect(isMethodAllowed('settings', method)).toBe(false);
    }
  });

  it('rechaza metodos que el motor no tiene', () => {
    expect(isMethodAllowed('graph', 'graph/windows')).toBe(false);
    expect(isMethodAllowed('graph', '$/cancel')).toBe(false);
    expect(isMethodAllowed('graph', '__proto__')).toBe(false);
  });

  it('ninguna superficie incluye un metodo del host ni engine/info', () => {
    const hostOnly = new Set<string>([...HOST_METHODS, 'engine/info']);
    for (const surface of SURFACES) {
      for (const method of SURFACE_METHODS[surface]) expect(hostOnly.has(method)).toBe(false);
    }
  });
});

describe('ids de peticion del host', () => {
  it('cuenta desde 1', () => {
    const ids = createRequestIds();
    expect([ids.next(), ids.next(), ids.next()]).toEqual([1, 2, 3]);
  });

  it('vuelve a 1 al llegar al maximo del protocolo', () => {
    const ids = createRequestIds(MAX_REQUEST_ID - 1);
    expect([ids.next(), ids.next()]).toEqual([MAX_REQUEST_ID, 1]);
  });
});
