import type { GraphRow } from '@git-rudder/bridge';
import { laneColor } from '@git-rudder/ui';
import type { ReactElement } from 'react';

import type { GraphMetrics } from '../graph/geometry';
import { type EdgeEntry, type NodeShape, type RowRange, layoutRange } from '../graph/layout';

import { moduleClasses } from './css';
import styles from './graph-svg.module.css';

const css = moduleClasses(styles);

export interface GraphSvgProps {
  readonly rows: readonly GraphRow[];
  readonly edges: readonly EdgeEntry[];
  /** Filas que pinta el virtualizador, ambas incluidas. */
  readonly range: RowRange;
  readonly metrics: GraphMetrics;
  readonly selectedIndex: number | null;
  /** X de la columna del grafo dentro de la fila (el ancho de la de refs). */
  readonly left: number;
  readonly width: number;
}

/**
 * Un solo SVG para las filas visibles, encima de la columna del grafo. Los
 * caminos van en coordenadas absolutas (fila * alto) y el viewBox recorta el
 * tramo que se ve, asi un camino largo se pinta una vez y no una por fila.
 * Es decorativo para los lectores de pantalla: la informacion esta en las filas.
 */
export function GraphSvg({
  rows,
  edges,
  range,
  metrics,
  selectedIndex,
  left,
  width,
}: GraphSvgProps): ReactElement {
  const shapes = layoutRange(rows, edges, range, metrics, selectedIndex);
  const top = range.start * metrics.rowHeight;
  const height = (range.end - range.start + 1) * metrics.rowHeight;
  return (
    <svg
      className={css('graph')}
      aria-hidden="true"
      focusable="false"
      width={width}
      height={height}
      viewBox={`0 ${String(top)} ${String(width)} ${String(height)}`}
      style={{ top, left }}
    >
      {shapes.edges.map((edge) => (
        <path
          key={edge.key}
          className={css('edge')}
          d={edge.d}
          style={{ stroke: laneColor(edge.lane) }}
        />
      ))}
      {shapes.nodes.map((node) => (
        <GraphNode key={node.key} node={node} ringOffset={metrics.selectedRingOffset} />
      ))}
    </svg>
  );
}

function GraphNode({
  node,
  ringOffset,
}: {
  readonly node: NodeShape;
  readonly ringOffset: number;
}): ReactElement {
  const color = laneColor(node.lane);
  return (
    <g data-sha={node.key} data-selected={node.selected}>
      {node.selected ? (
        <circle
          className={css('ring')}
          cx={node.cx}
          cy={node.cy}
          r={node.r + ringOffset}
          style={{ stroke: color }}
        />
      ) : null}
      <circle
        className={css('node')}
        cx={node.cx}
        cy={node.cy}
        r={node.r}
        style={{ fill: color }}
      />
      {node.initials === null ? null : (
        <text className={css('initials')} x={node.cx} y={node.cy + 0.5}>
          {node.initials}
        </text>
      )}
    </g>
  );
}
