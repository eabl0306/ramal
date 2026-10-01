import type { GraphRow } from '@ramal/bridge';
import { describe, expect, it } from 'vitest';

import { COMFORTABLE, graphEdge, graphRow } from '../test/fixtures';

import { indexEdges, layoutRange } from './layout';

/**
 * El ejemplo resuelto de data-and-algorithms 3.5, con los carriles y los
 * carriles de viaje que da el algoritmo del motor:
 * carriles M0 F1 A0 E1 H2 B0 X1 N2 Q3 R0.
 */
function workedExample(): GraphRow[] {
  const row = (
    sha: string,
    lane: number,
    edges: GraphRow['edges'],
    authorName = 'Marta Rios',
  ): GraphRow => graphRow(sha, { lane, edges, parents: edges.map((e) => e.parentSha), authorName });
  return [
    row('M', 0, [
      graphEdge('A', 2, { from: 0 }),
      graphEdge('F', 1, { from: 0, travel: 1 }, 'merge'),
    ]),
    row('F', 1, [graphEdge('E', 3, { from: 1 })]),
    row('A', 0, [graphEdge('B', 5, { from: 0 })]),
    row('E', 1, [graphEdge('B', 5, { from: 1, to: 0 })]),
    row('H', 2, [graphEdge('B', 5, { from: 2, to: 0 })]),
    row('B', 0, [graphEdge('R', 9, { from: 0 })]),
    row('X', 1, [graphEdge('R', 9, { from: 1, to: 0 })]),
    row('N', 2, [
      graphEdge('R', 9, { from: 2, to: 0 }),
      graphEdge('Q', 8, { from: 2, travel: 3 }, 'merge'),
    ]),
    row('Q', 3, [graphEdge('R', 9, { from: 3, to: 0 })]),
    row('R', 0, [], 'ezequiel'),
  ];
}

describe('layoutRange sobre el ejemplo resuelto', () => {
  const rows = workedExample();
  const edges = indexEdges(rows);

  it('pinta todas las aristas y todos los nodos cuando se ve todo', () => {
    const shapes = layoutRange(rows, edges, { start: 0, end: 9 }, COMFORTABLE, null);

    expect(shapes.edges).toHaveLength(11);
    expect(shapes.nodes.map((node) => node.lane)).toEqual([0, 1, 0, 1, 2, 0, 1, 2, 3, 0]);
    expect(shapes.edges.find((edge) => edge.key === 'M:1')).toEqual({
      key: 'M:1',
      d: 'M16 20 C16 42 36 38 36 60 L36 60',
      lane: 1,
    });
    expect(shapes.edges.find((edge) => edge.key === 'E:0')?.d).toBe(
      'M36 140 L36 180 C36 202 16 198 16 220',
    );
  });

  it('da radio de merge y sin iniciales a los merges, e iniciales al resto', () => {
    const { nodes } = layoutRange(rows, edges, { start: 0, end: 9 }, COMFORTABLE, 9);

    expect(nodes[0]).toMatchObject({ key: 'M', cx: 16, cy: 20, r: 5, initials: null });
    expect(nodes[1]).toMatchObject({ key: 'F', cx: 36, cy: 60, r: 9, initials: 'MR' });
    expect(nodes[9]).toMatchObject({ key: 'R', initials: 'EZ', selected: true });
    expect(nodes.filter((node) => node.selected)).toHaveLength(1);
  });

  it('en un tramo intermedio pinta las aristas que lo cruzan aunque el hijo este arriba', () => {
    const shapes = layoutRange(rows, edges, { start: 6, end: 7 }, COMFORTABLE, null);

    expect(shapes.edges.map((edge) => edge.key)).toEqual(['B:0', 'X:0', 'N:0', 'N:1']);
    expect(shapes.nodes.map((node) => node.key)).toEqual(['X', 'N']);
  });

  it('recorta el rango a las filas que existen', () => {
    const shapes = layoutRange(rows, edges, { start: -3, end: 40 }, COMFORTABLE, null);

    expect(shapes.nodes).toHaveLength(10);
  });
});

describe('indexEdges', () => {
  it('resuelve un padre que llego en una pagina posterior por su sha', () => {
    const rows = [
      graphRow('a', { lane: 1, edges: [graphEdge('b', null, { from: 1 })] }),
      graphRow('b', { lane: 0 }),
    ];

    expect(indexEdges(rows)).toEqual([
      { key: 'a:0', childRow: 0, parentRow: 1, fromLane: 1, travelLane: 1, toLane: 0 },
    ]);
  });

  it('deja en null el padre que no esta cargado y la arista baja hasta el borde', () => {
    const rows = [
      graphRow('a', { edges: [graphEdge('z', null, { from: 0 })] }),
      graphRow('b', { lane: 1 }),
    ];
    const edges = indexEdges(rows);

    expect(edges[0]?.parentRow).toBeNull();
    expect(layoutRange(rows, edges, { start: 0, end: 1 }, COMFORTABLE, null).edges).toEqual([
      { key: 'a:0', d: 'M16 20 L16 80', lane: 0 },
    ]);
  });

  it('respeta la fila que manda el motor aunque el sha se repita', () => {
    const rows = [
      graphRow('a', { edges: [graphEdge('b', 2, { from: 0 })] }),
      graphRow('b', { lane: 1 }),
      graphRow('b', { lane: 0 }),
    ];

    expect(indexEdges(rows)[0]).toMatchObject({ parentRow: 2, toLane: 0 });
  });
});
