import type { PlainKey } from '@ramal/i18n';
import { Button, useT } from '@ramal/ui';
import type { ReactElement } from 'react';

import { COLUMN_COUNT } from '../graph/columns';

import { moduleClasses } from './css';
import styles from './graph-grid.module.css';

const css = moduleClasses(styles);

/** Las columnas, en orden (COLUMN_COUNT); el texto de cada cabecera sale de @ramal/i18n. */
const COLUMN_KEYS: readonly PlainKey[] = [
  'graph.column.refs',
  'graph.column.graph',
  'graph.column.message',
  'graph.column.author',
  'graph.column.date',
  'graph.column.sha',
];

/** Fila de cabeceras, fija arriba al desplazar (es la fila 1 de la tabla). */
export function GraphHeader(): ReactElement {
  const t = useT();
  return (
    <div role="rowgroup" className={css('header-group')}>
      <div role="row" aria-rowindex={1} className={css('header')}>
        {COLUMN_KEYS.map((key) => (
          <div key={key} role="columnheader" className={css('header-cell')}>
            {t(key)}
          </div>
        ))}
      </div>
    </div>
  );
}

export interface LoaderRowProps {
  readonly index: number;
  readonly top: number;
  /** Fallo al pedir la pagina siguiente, o `null` mientras se pide. */
  readonly error: unknown;
  readonly errorMessage: string;
  readonly onRetry: () => void;
}

/**
 * Ultima fila mientras quedan commits: dice que se esta cargando la pagina
 * siguiente o, si fallo, por que, con un boton para reintentar.
 */
export function LoaderRow({
  index,
  top,
  error,
  errorMessage,
  onRetry,
}: LoaderRowProps): ReactElement {
  const t = useT();
  return (
    <div
      role="row"
      aria-rowindex={index + 2}
      className={css('row')}
      style={{ transform: `translateY(${String(top)}px)` }}
    >
      <div role="gridcell" aria-colspan={COLUMN_COUNT} className={css('loader')}>
        {error === null ? (
          <span role="status">{t('graph.view.loadingMore')}</span>
        ) : (
          <>
            <span role="alert">{errorMessage}</span>
            <Button size={30} variant="ghost" onPress={onRetry}>
              {t('graph.view.retry')}
            </Button>
          </>
        )}
      </div>
    </div>
  );
}
