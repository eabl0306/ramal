import type { GraphEdge, GraphRow, GraphWindow, RefLabel } from '@ramal/bridge';
import { DENSITIES } from '@ramal/ui';

import type { GraphMetrics } from '../graph/geometry';

/** Geometria de escritorio y de VS Code, la que usa la vista con cada densidad. */
export const COMFORTABLE: GraphMetrics = {
  ...DENSITIES.comfortable.graph,
  rowHeight: DENSITIES.comfortable.rowHeight,
};

export const COMPACT: GraphMetrics = {
  ...DENSITIES.compact.graph,
  rowHeight: DENSITIES.compact.rowHeight,
};

/** Fila con valores razonables; cada test cambia lo que le importa. */
export function graphRow(sha: string, overrides: Partial<GraphRow> = {}): GraphRow {
  return {
    sha,
    shortSha: sha.slice(0, 7),
    parents: [],
    lane: 0,
    edges: [],
    summary: `Commit ${sha}`,
    authorName: 'Ana Ruiz',
    authorEmail: 'ana@example.com',
    authoredAt: '2026-04-12T10:24:00+02:00',
    refs: [],
    isHead: false,
    ...overrides,
  };
}

/** Arista con el padre en la fila dada (o fuera de lo cargado con `null`). */
export function graphEdge(
  parentSha: string,
  parentRow: number | null,
  lanes: { readonly from: number; readonly travel?: number; readonly to?: number },
  kind: GraphEdge['kind'] = 'firstParent',
): GraphEdge {
  const travelLane = lanes.travel ?? lanes.from;
  return {
    parentSha,
    parentRow,
    fromLane: lanes.from,
    travelLane,
    toLane: lanes.to ?? travelLane,
    kind,
  };
}

export function refLabel(name: string, overrides: Partial<RefLabel> = {}): RefLabel {
  return {
    name,
    fullName: `refs/heads/${name}`,
    kind: 'localBranch',
    isHead: false,
    ...overrides,
  };
}

/** Historial lineal de `count` commits en el carril 0, del mas nuevo al mas viejo. */
export function linearRows(count: number, offset = 0): GraphRow[] {
  return Array.from({ length: count }, (_, index) => {
    const row = offset + index;
    const parent = `c${String(row + 1)}`;
    return graphRow(`c${String(row)}`, {
      parents: [parent],
      edges: [graphEdge(parent, row + 1, { from: 0 })],
    });
  });
}

/** Ventana del motor sobre unas filas. */
export function graphWindow(
  rows: readonly GraphRow[],
  offset: number,
  hasMore: boolean,
  laneCount = 1,
): GraphWindow {
  return { repoId: 'repo-1', offset, rows: [...rows], hasMore, laneCount, snapshot: 'snap-1' };
}
