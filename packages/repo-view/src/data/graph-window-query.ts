import {
  type GraphRow,
  type GraphWindow,
  type GraphWindowParams,
  isBridgeError,
} from '@ramal/bridge';

/**
 * Lo puro de la consulta `graph/window`: claves, paginas y reintentos. El hook
 * solo cablea esto con TanStack Query, asi que se prueba sin React.
 */

/**
 * Filas por peticion. 200 llena varias pantallas en las dos densidades sin que
 * la respuesta pese: el motor admite hasta 1000, pero mas filas por peticion no
 * pintan antes, solo alargan la primera espera.
 */
export const DEFAULT_PAGE_SIZE = 200;

/** Clave de la consulta: un repositorio y un tamano de pagina, una cache. */
export type GraphWindowKey = readonly ['repo-view', 'graph-window', string, number];

/**
 * Empieza por 'repo-view' para que un host pueda invalidar todo lo del paquete
 * de una vez cuando el motor avise de cambios (planificado: `repo/changed`).
 */
export function graphWindowKey(repoId: string, pageSize: number): GraphWindowKey {
  return ['repo-view', 'graph-window', repoId, pageSize];
}

/**
 * Que pagina pedir: la fila donde empieza y la huella de la primera pagina. La
 * primera no lleva huella; las siguientes la repiten para que el motor diga
 * `stale_window` si el repositorio cambio entre medias.
 */
export interface GraphPageParam {
  readonly offset: number;
  readonly snapshot?: string;
}

export const FIRST_PAGE: GraphPageParam = { offset: 0 };

/**
 * Pagina siguiente, o `undefined` si no hay mas. Una pagina vacia que dice tener
 * mas cortaria el bucle de todos modos: pedir otra vez el mismo offset no
 * avanzaria nunca.
 */
export function nextPageParam(last: GraphWindow): GraphPageParam | undefined {
  if (!last.hasMore || last.rows.length === 0) return undefined;
  return { offset: last.offset + last.rows.length, snapshot: last.snapshot };
}

/** Parametros de `graph/window` para una pagina. */
export function windowParams(
  repoId: string,
  limit: number,
  page: GraphPageParam,
): GraphWindowParams {
  const params = { repoId, offset: page.offset, limit };
  return page.snapshot === undefined ? params : { ...params, snapshot: page.snapshot };
}

/** Las paginas cargadas, unidas en una sola lista de filas absolutas. */
export interface LoadedGraph {
  readonly rows: readonly GraphRow[];
  /** Carriles que necesita la columna; el mayor de las paginas cargadas. */
  readonly laneCount: number;
}

export function mergePages(pages: readonly GraphWindow[]): LoadedGraph {
  return {
    rows: pages.flatMap((page) => page.rows),
    laneCount: pages.reduce((max, page) => Math.max(max, page.laneCount), 0),
  };
}

/**
 * El repositorio cambio entre dos paginas (un commit o un fetch desde fuera).
 * No es un fallo que ensenar ni que reintentar tal cual: se vuelve a empezar.
 */
export function isStaleWindow(error: unknown): boolean {
  return isBridgeError(error) && error.kind === 'stale_window';
}

/**
 * Errores que merecen otro intento: los del camino hasta el motor, que pueden
 * ser pasajeros (el sidecar reiniciandose, un timeout). Un error del motor
 * (repo no abierto, git que falla) se repetiria igual, y `stale_window` se
 * resuelve volviendo a la primera pagina, no repitiendo la misma peticion.
 */
const TRANSIENT_KINDS = new Set(['transport', 'timeout', 'engine_unavailable']);

/** Reintentos como mucho ante un error pasajero. */
export const MAX_TRANSIENT_RETRIES = 2;

export function shouldRetry(failureCount: number, error: unknown): boolean {
  return (
    failureCount < MAX_TRANSIENT_RETRIES && isBridgeError(error) && TRANSIENT_KINDS.has(error.kind)
  );
}
