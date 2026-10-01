import type { GraphRow } from '@ramal/bridge';
import { type Density, useT } from '@ramal/ui';
import type { VirtualItem } from '@tanstack/react-virtual';
import { type CSSProperties, type ReactElement, useId, useMemo, useRef } from 'react';

import { engineErrorKey } from '../data/engine-error';
import type { GraphWindowState } from '../data/use-graph-window';
import { COLUMN_COUNT } from '../graph/columns';
import { indexEdges } from '../graph/layout';
import { type ViewMetrics, viewMetrics } from '../graph/view-metrics';

import { cssVars, moduleClasses } from './css';
import styles from './graph-grid.module.css';
import { GraphHeader, LoaderRow } from './graph-header';
import { GraphRowView } from './graph-row';
import { GraphSvg } from './graph-svg';
import { useGridInput, useGridSelection } from './use-grid-selection';
import { gridScroller, useAutoLoad, useGridVirtualizer } from './use-grid-virtualizer';

const css = moduleClasses(styles);

export interface GraphGridProps {
  readonly graph: GraphWindowState;
  readonly density: Density;
  readonly selectedSha?: string | null | undefined;
  readonly onSelectionChange?: ((row: GraphRow) => void) | undefined;
  readonly onActivate?: ((row: GraphRow) => void) | undefined;
}

/**
 * La tabla del grafo con filas virtualizadas: solo existen en el DOM las filas
 * que se ven (mas un margen), y un unico SVG pinta los carriles de ese tramo.
 * Es un grid de ARIA: cabeceras de columna, `aria-rowcount` (-1 mientras quedan
 * paginas, porque el total no se sabe) y la fila activa por aria-activedescendant,
 * que solo apunta a la fila si esta montada.
 */
export function GraphGrid(props: GraphGridProps): ReactElement {
  const { graph, density } = props;
  const t = useT();
  const baseId = useId();
  const scrollRef = useRef<HTMLDivElement>(null);
  const view = useMemo(() => viewMetrics(density, graph.laneCount), [density, graph.laneCount]);
  const virtualizer = useGridVirtualizer(scrollRef, graph, view);
  const selection = useGridSelection({
    rows: graph.rows,
    selectedSha: props.selectedSha,
    onSelectionChange: props.onSelectionChange,
  });
  const input = useGridInput({
    rows: graph.rows,
    selection,
    onActivate: props.onActivate,
    ...gridScroller(virtualizer, view),
  });
  const items = virtualizer.getVirtualItems();
  useAutoLoad(items.at(-1)?.index ?? -1, graph);
  const rowId = (index: number): string => `${baseId}-row-${String(index)}`;
  const body = { graph, view, items, rowId, selectedIndex: selection.selectedIndex };
  return (
    <div
      ref={scrollRef}
      role="grid"
      tabIndex={0}
      aria-label={t('graph.view.label')}
      aria-rowcount={graph.hasMore ? -1 : graph.rows.length + 1}
      aria-colcount={COLUMN_COUNT}
      aria-activedescendant={activeRowId(body)}
      data-density={density}
      className={css('grid')}
      style={gridStyle(view)}
      {...input}
    >
      <GraphHeader />
      <div className={css('body')} style={{ height: virtualizer.getTotalSize() }}>
        <GridRows {...body} />
        <GridGraph {...body} />
      </div>
    </div>
  );
}

/** Id de la fila seleccionada, solo si esta montada: aria-activedescendant no puede apuntar al vacio. */
function activeRowId({ items, rowId, selectedIndex }: GridRowsProps): string | undefined {
  if (selectedIndex === null) return undefined;
  return items.some((item) => item.index === selectedIndex) ? rowId(selectedIndex) : undefined;
}

/** Las medidas que comparten cabecera y filas, como variables CSS. */
function gridStyle(view: ViewMetrics): CSSProperties {
  return cssVars({
    '--rv-columns': view.template,
    '--rv-min-width': `${String(view.columns.minWidth)}px`,
    '--rv-header-h': `${String(view.columns.headerHeight)}px`,
  });
}

/**
 * Las partes del cuerpo reciben las filas virtuales ya calculadas y no el
 * virtualizador: dependen solo de valores que cambian cuando hay que pintar.
 */
interface BodyPartProps {
  readonly graph: GraphWindowState;
  readonly view: ViewMetrics;
  readonly items: readonly VirtualItem[];
  readonly selectedIndex: number | null;
}

interface GridRowsProps extends BodyPartProps {
  readonly rowId: (index: number) => string;
}

/** Las filas montadas y, al final, la de carga mientras quedan paginas. */
function GridRows({ graph, view, items, rowId, selectedIndex }: GridRowsProps): ReactElement {
  const t = useT();
  const offset = view.columns.headerHeight;
  return (
    <div role="rowgroup" className={css('rows')}>
      {items.map((item) => {
        const row = graph.rows[item.index];
        const top = item.start - offset;
        return row === undefined ? (
          <LoaderRow
            key="loader"
            index={item.index}
            top={top}
            error={graph.loadMoreError}
            errorMessage={t(engineErrorKey(graph.loadMoreError))}
            onRetry={graph.retry}
          />
        ) : (
          <GraphRowView
            key={row.sha}
            row={row}
            index={item.index}
            id={rowId(item.index)}
            selected={item.index === selectedIndex}
            top={top}
          />
        );
      })}
    </div>
  );
}

/** El SVG de los carriles sobre las filas montadas que son commits. */
function GridGraph({ graph, view, items, selectedIndex }: BodyPartProps): ReactElement | null {
  const edges = useMemo(() => indexEdges(graph.rows), [graph.rows]);
  const start = items[0]?.index ?? 0;
  const end = Math.min(items.at(-1)?.index ?? -1, graph.rows.length - 1);
  if (start > end) return null;
  return (
    <GraphSvg
      rows={graph.rows}
      edges={edges}
      range={{ start, end }}
      metrics={view.metrics}
      selectedIndex={selectedIndex}
      left={view.columns.refs}
      width={view.graphWidth}
    />
  );
}
