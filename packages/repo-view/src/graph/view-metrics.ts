import { DENSITIES, type Density } from '@git-rudder/ui';

import { COLUMN_LAYOUTS, type ColumnLayout, gridTemplate } from './columns';
import { type GraphMetrics, graphColumnWidth } from './geometry';

/** Todo lo que la vista mide en px para una densidad y un numero de carriles. */
export interface ViewMetrics {
  readonly metrics: GraphMetrics;
  readonly columns: ColumnLayout;
  /** Ancho real de la columna del grafo. */
  readonly graphWidth: number;
  /** `grid-template-columns` comun a la cabecera y a las filas. */
  readonly template: string;
}

/**
 * Los numeros salen de `DENSITIES` de @git-rudder/ui, que un test compara con los
 * tokens CSS: el alto de fila del SVG y el de las filas no pueden separarse. La
 * densidad la pasa el host (comfortable en escritorio, compact en VS Code); no
 * hay ramas por host en los componentes.
 */
export function viewMetrics(density: Density, laneCount: number): ViewMetrics {
  const { graph, rowHeight } = DENSITIES[density];
  const metrics: GraphMetrics = { ...graph, rowHeight };
  const columns = COLUMN_LAYOUTS[density];
  const graphWidth = graphColumnWidth(laneCount, metrics, columns.graphMin);
  return { metrics, columns, graphWidth, template: gridTemplate(columns, graphWidth) };
}
