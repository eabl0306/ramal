import { type GraphMetrics, laneX, rowY } from './geometry';

/**
 * Una arista ya resuelta a filas absolutas. `parentRow` es `null` cuando el
 * padre no esta entre las filas cargadas: la arista baja entonces hasta el borde
 * inferior (gaps B-16; el prototipo la omitia).
 */
export interface EdgeSpan {
  readonly childRow: number;
  readonly parentRow: number | null;
  readonly fromLane: number;
  readonly travelLane: number;
  readonly toLane: number;
}

/**
 * Camino SVG de una arista en tres tramos, como lo calcula el motor (gaps B-15):
 *
 * 1. Curva en la fila del hijo de `fromLane` a `travelLane`, si son distintos.
 *    Es la curva de merge del diseno ("curva en la primera fila").
 * 2. Vertical por `travelLane`, el hueco donde vivio la reserva: nadie mas lo
 *    ocupa entre las dos filas, asi la linea no cruza nodos ajenos.
 * 3. Curva en la fila del padre de `travelLane` a `toLane`, si son distintos.
 *    Es la curva de rama que sale del diseno ("vertical y curva en la ultima
 *    fila").
 *
 * Con una arista de primer padre el carril de viaje es el del hijo, asi que las
 * formas del diseno salen igual: recta en el mismo carril, vertical y curva al
 * salir una rama, curva y vertical en un merge. Los puntos de control son los de
 * la densidad (22 y 18 px de 40 en escritorio, 16 y 12 de 28 en VS Code).
 *
 * El formato del texto (`M16 20 C16 42 36 38 36 60 L36 60`) es el del prototipo:
 * los tests comparan los caminos exactos de data-and-algorithms 3.5.
 */
export function edgePath(span: EdgeSpan, metrics: GraphMetrics, bottomY: number): string {
  const x1 = laneX(span.fromLane, metrics);
  const xt = laneX(span.travelLane, metrics);
  const x2 = laneX(span.toLane, metrics);
  const y1 = rowY(span.childRow, metrics);
  const y2 = span.parentRow === null ? bottomY : rowY(span.parentRow, metrics);
  const firstCurve = span.fromLane !== span.travelLane;
  const lastCurve = span.parentRow !== null && span.toLane !== span.travelLane;
  const rh = metrics.rowHeight;

  // Con el padre en la fila siguiente no caben dos curvas: una sola va del
  // carril del hijo al del padre.
  if (firstCurve && lastCurve && y2 - y1 < 2 * rh) {
    return `M${n(x1)} ${n(y1)} ${curveDown(x1, x2, y1, metrics)} L${n(x2)} ${n(y2)}`;
  }
  const parts = [`M${n(x1)} ${n(y1)}`];
  if (firstCurve) parts.push(curveDown(x1, xt, y1, metrics));
  parts.push(`L${n(xt)} ${n(lastCurve ? y2 - rh : y2)}`);
  if (lastCurve) parts.push(curveUp(xt, x2, y2, metrics));
  return parts.join(' ');
}

/** Curva que baja una fila desde `y` cambiando de carril. */
function curveDown(from: number, to: number, y: number, metrics: GraphMetrics): string {
  const { controlNear, controlFar, rowHeight } = metrics;
  return `C${n(from)} ${n(y + controlNear)} ${n(to)} ${n(y + controlFar)} ${n(to)} ${n(y + rowHeight)}`;
}

/** Curva que llega a `y` desde la fila de arriba cambiando de carril. */
function curveUp(from: number, to: number, y: number, metrics: GraphMetrics): string {
  const { controlNear, controlFar } = metrics;
  return `C${n(from)} ${n(y - controlFar)} ${n(to)} ${n(y - controlNear)} ${n(to)} ${n(y)}`;
}

/** Numero sin decimales de coma flotante sobrantes (0.1 + 0.2). */
function n(value: number): string {
  return String(Math.round(value * 100) / 100);
}
