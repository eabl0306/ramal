import { BRIDGE_ERROR_KINDS, BridgeError } from '@ramal/bridge';
import { createTranslator } from '@ramal/i18n';
import { describe, expect, it } from 'vitest';

import { graphWindow, linearRows } from '../test/fixtures';

import { ENGINE_ERROR_KEYS, engineErrorKey } from './engine-error';
import {
  FIRST_PAGE,
  MAX_TRANSIENT_RETRIES,
  graphWindowKey,
  isStaleWindow,
  mergePages,
  nextPageParam,
  shouldRetry,
  windowParams,
} from './graph-window-query';
import { createRepoViewQueryClient } from './query-client';

describe('paginas de graph/window', () => {
  it('la clave empieza por repo-view y separa repositorio y tamano de pagina', () => {
    expect(graphWindowKey('repo-1', 200)).toEqual(['repo-view', 'graph-window', 'repo-1', 200]);
  });

  it('pide la pagina siguiente justo despues de la ultima fila con la huella recibida', () => {
    expect(nextPageParam(graphWindow(linearRows(20), 40, true))).toEqual({
      offset: 60,
      snapshot: 'snap-1',
    });
  });

  it('no pide mas al final del historial ni tras una pagina vacia', () => {
    expect(nextPageParam(graphWindow(linearRows(5), 40, false))).toBeUndefined();
    expect(nextPageParam(graphWindow([], 40, true))).toBeUndefined();
  });

  it('la primera pagina va sin huella y las siguientes la repiten', () => {
    expect(windowParams('repo-1', 200, FIRST_PAGE)).toStrictEqual({
      repoId: 'repo-1',
      offset: 0,
      limit: 200,
    });
    expect(windowParams('repo-1', 200, { offset: 200, snapshot: 'abc' })).toStrictEqual({
      repoId: 'repo-1',
      offset: 200,
      limit: 200,
      snapshot: 'abc',
    });
  });

  it('une las paginas en filas absolutas y se queda con el mayor numero de carriles', () => {
    const merged = mergePages([
      graphWindow(linearRows(3), 0, true, 2),
      graphWindow(linearRows(2, 3), 3, false, 4),
    ]);

    expect(merged.rows.map((row) => row.sha)).toEqual(['c0', 'c1', 'c2', 'c3', 'c4']);
    expect(merged.laneCount).toBe(4);
    expect(mergePages([])).toEqual({ rows: [], laneCount: 0 });
  });
});

describe('shouldRetry', () => {
  it('reintenta los fallos del camino hasta el motor un numero limitado de veces', () => {
    for (const kind of ['transport', 'timeout', 'engine_unavailable'] as const) {
      expect(shouldRetry(0, new BridgeError(kind, 'x'))).toBe(true);
      expect(shouldRetry(MAX_TRANSIENT_RETRIES, new BridgeError(kind, 'x'))).toBe(false);
    }
  });

  it('no reintenta un error del motor ni algo que no es un BridgeError', () => {
    expect(shouldRetry(0, new BridgeError('repo_not_open', 'x'))).toBe(false);
    expect(shouldRetry(0, new BridgeError('stale_window', 'x'))).toBe(false);
    expect(shouldRetry(0, new Error('boom'))).toBe(false);
  });
});

describe('isStaleWindow', () => {
  it('reconoce solo el aviso de que el historial cambio entre dos paginas', () => {
    expect(isStaleWindow(new BridgeError('stale_window', 'x'))).toBe(true);
    expect(isStaleWindow(new BridgeError('git_failed', 'x'))).toBe(false);
    expect(isStaleWindow(new Error('stale_window'))).toBe(false);
    expect(isStaleWindow(null)).toBe(false);
  });
});

describe('textos de error', () => {
  it('cada tipo de error del motor y del bridge tiene un texto traducido', () => {
    const t = createTranslator('es');
    for (const kind of BRIDGE_ERROR_KINDS) {
      const key = ENGINE_ERROR_KEYS[kind];
      expect(t(key)).not.toBe(key);
    }
  });

  it('traduce el kind de un BridgeError y trata lo demas como interno', () => {
    expect(engineErrorKey(new BridgeError('git_not_found', 'x'))).toBe(
      'errors.engine.git_not_found',
    );
    expect(engineErrorKey(new TypeError('boom'))).toBe('errors.engine.internal');
  });
});

describe('createRepoViewQueryClient', () => {
  it('no reintenta ni refresca por foco o por red por defecto', () => {
    const queries = createRepoViewQueryClient().getDefaultOptions().queries;

    expect(queries).toMatchObject({
      retry: false,
      refetchOnWindowFocus: false,
      refetchOnReconnect: false,
    });
  });
});
