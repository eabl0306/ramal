import { type GraphRow, type GraphWindow, isBridgeError } from '@ramal/bridge';

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
 * Offset de la pagina siguiente, o `undefined` si no hay mas. Una pagina vacia
 * que dice tener mas cortaria el bucle de todos modos: pedir otra vez el mismo
 * offset no avanzaria nunca.
 */
export function nextOffset(last: GraphWindow): number | undefined {
  if (!last.hasMore || last.rows.length === 0) return undefined;
  return last.offset + last.rows.length;
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
 * Errores que merecen otro intento: los del camino hasta el motor, que pueden
 * ser pasajeros (el sidecar reiniciandose, un timeout). Un error del motor
 * (repo no abierto, git que falla) se repetiria igual.
 */
const TRANSIENT_KINDS = new Set(['transport', 'timeout', 'engine_unavailable']);

/** Reintentos como mucho ante un error pasajero. */
export const MAX_TRANSIENT_RETRIES = 2;

export function shouldRetry(failureCount: number, error: unknown): boolean {
  return (
    failureCount < MAX_TRANSIENT_RETRIES && isBridgeError(error) && TRANSIENT_KINDS.has(error.kind)
  );
}
