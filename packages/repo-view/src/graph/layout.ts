import type { GraphRow } from '@git-rudder/bridge';

import { type EdgeSpan, edgePath } from './edge-path';
import { type GraphMetrics, laneX, rowY } from './geometry';
import { authorInitials } from './initials';

/** Una arista con su clave estable y las filas ya resueltas. */
export interface EdgeEntry extends EdgeSpan {
  readonly key: string;
}

/** Arista lista para pintar: camino y carril del color. */
export interface EdgeShape {
  readonly key: string;
  readonly d: string;
  readonly lane: number;
}

/** Nodo listo para pintar. `initials` es `null` en los merges, que no las llevan. */
export interface NodeShape {
  readonly key: string;
  readonly cx: number;
  readonly cy: number;
  readonly r: number;
  readonly lane: number;
  readonly initials: string | null;
  readonly selected: boolean;
}

export interface GraphShapes {
  readonly edges: readonly EdgeShape[];
  readonly nodes: readonly NodeShape[];
}

/** Filas visibles, ambas incluidas. */
export interface RowRange {
  readonly start: number;
  readonly end: number;
}

/**
 * Todas las aristas de las filas cargadas, en orden de fila del hijo.
 *
 * El motor manda `parentRow: null` cuando el padre cae fuera de lo que leyo
 * para esa ventana. Al cargar la pagina siguiente el padre puede aparecer: se
 * busca su sha entre las filas cargadas y, si esta, la arista llega a su fila y
 * a su carril. Si no esta, sigue bajando hasta el borde inferior.
 */
export function indexEdges(rows: readonly GraphRow[]): EdgeEntry[] {
  const rowBySha = new Map<string, number>();
  rows.forEach((row, index) => {
    if (!rowBySha.has(row.sha)) rowBySha.set(row.sha, index);
  });
  return rows.flatMap((row, childRow) =>
    row.edges.map((edge, position) => {
      const parentRow = edge.parentRow ?? rowBySha.get(edge.parentSha) ?? null;
      const parentLane = parentRow === null ? undefined : rows[parentRow]?.lane;
      return {
        key: `${row.sha}:${String(position)}`,
        childRow,
        parentRow,
        fromLane: edge.fromLane,
        travelLane: edge.travelLane,
        toLane: edge.parentRow === null ? (parentLane ?? edge.toLane) : edge.toLane,
      };
    }),
  );
}

/**
 * Lo que hay que pintar en las filas visibles: las aristas que las cruzan
 * (aunque el hijo este mas arriba) y los nodos de esas filas. Las aristas van
 * primero y los nodos encima, como en el prototipo.
 *
 * El color de una arista es el de su carril de viaje: en una de primer padre es
 * el del hijo y en un merge el de la rama que entra, que es lo que pide el
 * diseno, y ademas el tramo vertical siempre tiene el color de su carril.
 */
export function layoutRange(
  rows: readonly GraphRow[],
  edges: readonly EdgeEntry[],
  range: RowRange,
  metrics: GraphMetrics,
  selectedIndex: number | null,
): GraphShapes {
  const bottomY = rows.length * metrics.rowHeight;
  const visibleEdges: EdgeShape[] = [];
  for (const edge of edges) {
    if (edge.childRow > range.end) break;
    if ((edge.parentRow ?? Number.POSITIVE_INFINITY) < range.start) continue;
    visibleEdges.push({
      key: edge.key,
      d: edgePath(edge, metrics, bottomY),
      lane: edge.travelLane,
    });
  }
  const nodes: NodeShape[] = [];
  const last = Math.min(range.end, rows.length - 1);
  for (let index = Math.max(range.start, 0); index <= last; index += 1) {
    const row = rows[index];
    if (row !== undefined) nodes.push(nodeShape(row, index, metrics, index === selectedIndex));
  }
  return { edges: visibleEdges, nodes };
}

function nodeShape(
  row: GraphRow,
  index: number,
  metrics: GraphMetrics,
  selected: boolean,
): NodeShape {
  const isMerge = row.parents.length > 1;
  return {
    key: row.sha,
    cx: laneX(row.lane, metrics),
    cy: rowY(index, metrics),
    r: isMerge ? metrics.mergeRadius : metrics.nodeRadius,
    lane: row.lane,
    initials: isMerge ? null : authorInitials(row.authorName),
    selected,
  };
}
