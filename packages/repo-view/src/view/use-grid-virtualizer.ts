import { type Virtualizer, useVirtualizer } from '@tanstack/react-virtual';
import { type RefObject, useEffect } from 'react';

import type { GraphWindowState } from '../data/use-graph-window';
import type { ViewMetrics } from '../graph/view-metrics';

/** Filas que se pintan fuera de la pantalla, arriba y abajo, para que el scroll no parpadee. */
const OVERSCAN = 12;
/** A cuantas filas del final se pide la pagina siguiente. */
const PREFETCH_ROWS = 60;

export type GridVirtualizer = Virtualizer<HTMLDivElement, Element>;

/**
 * Virtualizador sobre la propia tabla, que es la que desplaza. La cabecera fija
 * ocupa el principio del contenido: `scrollMargin` dice donde empiezan las filas
 * y `scrollPaddingStart` evita que una fila quede debajo de la cabecera al
 * llevarla a la vista con el teclado. Sin flushSync: React 19 avisa si se usa
 * durante el renderizado y con filas de alto fijo no aporta nada. Hay una fila
 * mas al final mientras quedan paginas: la que dice que se esta cargando.
 */
export function useGridVirtualizer(
  scrollRef: RefObject<HTMLDivElement | null>,
  graph: GraphWindowState,
  view: ViewMetrics,
): GridVirtualizer {
  const rowHeight = view.metrics.rowHeight;
  // eslint-disable-next-line react-hooks/incompatible-library -- el aviso es para React Compiler, que ningun host usa; aun asi, lo que sale de aqui hacia componentes memorizables son valores nuevos en cada cambio (las filas virtuales y el alto total), nunca el virtualizador
  const virtualizer = useVirtualizer({
    count: graph.rows.length + (graph.hasMore ? 1 : 0),
    getScrollElement: () => scrollRef.current,
    estimateSize: () => rowHeight,
    overscan: OVERSCAN,
    scrollMargin: view.columns.headerHeight,
    scrollPaddingStart: view.columns.headerHeight,
    useFlushSync: false,
  });
  // Con otra densidad cambia el alto de fila: hay que olvidar las medidas viejas.
  useEffect(() => {
    virtualizer.measure();
  }, [virtualizer, rowHeight]);
  return virtualizer;
}

/** Como mover la vista desde el teclado: llevar una fila a la vista y medir una pagina. */
export interface GridScroller {
  readonly scrollToIndex: (index: number) => void;
  readonly pageRows: () => number;
}

export function gridScroller(virtualizer: GridVirtualizer, view: ViewMetrics): GridScroller {
  return {
    scrollToIndex: (index) => {
      virtualizer.scrollToIndex(index, { align: 'auto' });
    },
    // Filas que caben debajo de la cabecera; como poco una.
    pageRows: () => {
      const height = (virtualizer.scrollRect?.height ?? 0) - view.columns.headerHeight;
      return Math.max(1, Math.floor(height / view.metrics.rowHeight));
    },
  };
}

/**
 * Pide la pagina siguiente cuando las filas visibles se acercan al final de lo
 * cargado. Tras un fallo no insiste sola: espera a que la persona pulse
 * reintentar, o la tabla pediria la misma pagina en bucle.
 */
export function useAutoLoad(lastVisibleIndex: number, graph: GraphWindowState): void {
  const { rows, hasMore, isLoadingMore, loadMoreError, loadMore } = graph;
  const nearEnd = lastVisibleIndex >= rows.length - PREFETCH_ROWS;
  const blocked = loadMoreError !== null;
  useEffect(() => {
    if (nearEnd && hasMore && !isLoadingMore && !blocked) loadMore();
  }, [nearEnd, hasMore, isLoadingMore, blocked, loadMore]);
}
