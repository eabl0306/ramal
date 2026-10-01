import type { GraphGeometry } from '@ramal/ui';

/**
 * Geometria del SVG del grafo: la de `DENSITIES` de @ramal/ui mas el alto de
 * fila, que tambien sale de la densidad. Se recibe como parametro en lugar de
 * leer @ramal/ui aqui, asi estas funciones son puras y los tests fijan los
 * numeros del diseno sin montar React.
 */
export interface GraphMetrics extends GraphGeometry {
  readonly rowHeight: number;
}

/** Grosor del trazo de las aristas y del anillo de seleccion. */
export const STROKE_WIDTH = 2;

/** X del centro de un carril: `originX + lane * laneWidth` (16 + carril * 20 en escritorio). */
export function laneX(lane: number, metrics: GraphMetrics): number {
  return metrics.originX + lane * metrics.laneWidth;
}

/** Y del centro de una fila: `row * rowHeight + rowHeight / 2`. */
export function rowY(row: number, metrics: GraphMetrics): number {
  return row * metrics.rowHeight + metrics.rowHeight / 2;
}

/**
 * Ancho de la columna del grafo. Nunca menos que el del diseno (130 px en
 * escritorio, donde caben los carriles 0 a 5 con el anillo de seleccion), y mas
 * si el motor dice que hacen falta mas carriles: con el modulo de colores (gaps
 * B-14) los carriles siguen teniendo color, pero necesitan sitio.
 */
export function graphColumnWidth(
  laneCount: number,
  metrics: GraphMetrics,
  minimum: number,
): number {
  const lastLane = Math.max(laneCount - 1, 0);
  const needed =
    laneX(lastLane, metrics) + metrics.nodeRadius + metrics.selectedRingOffset + STROKE_WIDTH / 2;
  return Math.max(minimum, Math.ceil(needed));
}
