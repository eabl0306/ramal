import type { Density } from '@ramal/ui';

/** Refs, grafo, mensaje, autor, fecha y sha: las columnas de la tabla. */
export const COLUMN_COUNT = 6;

/**
 * Columnas de la tabla del grafo por densidad, en px. Escritorio (comfortable)
 * es `200 | 130 | 1fr | 150 | 110 | 76` del prototipo de escritorio; VS Code
 * (compact) es `130 | 120 | minmax(150px, 1fr) | 100 | 84 | 60` del suyo. Viven
 * en JavaScript y no en el CSS porque el SVG necesita el ancho de la columna de
 * refs para colocarse encima de la del grafo: asi hay una sola fuente.
 */
export interface ColumnLayout {
  readonly refs: number;
  /** Minimo de la columna del grafo; crece si hacen falta mas carriles. */
  readonly graphMin: number;
  /** Minimo de la columna del mensaje, que se queda con el resto. */
  readonly messageMin: number;
  readonly author: number;
  readonly date: number;
  readonly sha: number;
  /** Ancho por debajo del cual la tabla se desplaza en horizontal. */
  readonly minWidth: number;
  /** Alto de la fila de cabeceras. */
  readonly headerHeight: number;
}

export const COLUMN_LAYOUTS: Readonly<Record<Density, ColumnLayout>> = {
  comfortable: {
    refs: 200,
    graphMin: 130,
    messageMin: 0,
    author: 150,
    date: 110,
    sha: 76,
    minWidth: 760,
    headerHeight: 34,
  },
  compact: {
    refs: 130,
    graphMin: 120,
    messageMin: 150,
    author: 100,
    date: 84,
    sha: 60,
    minWidth: 560,
    headerHeight: 24,
  },
};

/** El `grid-template-columns` de cabecera y filas, con el ancho real del grafo. */
export function gridTemplate(layout: ColumnLayout, graphWidth: number): string {
  const px = (value: number): string => `${String(value)}px`;
  return [
    px(layout.refs),
    px(graphWidth),
    `minmax(${px(layout.messageMin)}, 1fr)`,
    px(layout.author),
    px(layout.date),
    px(layout.sha),
  ].join(' ');
}
