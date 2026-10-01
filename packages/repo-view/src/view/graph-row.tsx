import type { GraphRow } from '@ramal/bridge';
import { formatDate } from '@ramal/i18n';
import { laneColor, useLocale } from '@ramal/ui';
import { type ReactElement, memo } from 'react';

import { authorInitials } from '../graph/initials';

import { moduleClasses } from './css';
import styles from './graph-grid.module.css';
import { RefPills } from './ref-pills';

const css = moduleClasses(styles);

export interface GraphRowViewProps {
  readonly row: GraphRow;
  /** Fila absoluta: posicion, `aria-rowindex` y destino de los clics. */
  readonly index: number;
  readonly id: string;
  readonly selected: boolean;
  /** Desplazamiento vertical dentro del cuerpo de la tabla, en px. */
  readonly top: number;
}

/**
 * Una fila de la tabla: refs, hueco del grafo (lo pinta el SVG por encima),
 * mensaje, autor, fecha y sha corto. Los clics y el teclado los atiende la
 * tabla entera; la fila solo dice que indice es con `data-row-index`.
 */
export const GraphRowView = memo(function GraphRowView({
  row,
  index,
  id,
  selected,
  top,
}: GraphRowViewProps): ReactElement {
  return (
    <div
      role="row"
      id={id}
      aria-rowindex={index + 2}
      aria-selected={selected}
      data-row-index={index}
      className={css('row')}
      style={{ transform: `translateY(${String(top)}px)` }}
    >
      <RefPills refs={row.refs} lane={row.lane} />
      <div role="gridcell" />
      <div role="gridcell" className={css('cell')} title={row.summary}>
        <span className={css('message')} data-head={row.isHead}>
          {row.summary}
        </span>
      </div>
      <AuthorCell row={row} />
      <DateCell iso={row.authoredAt} />
      <div role="gridcell" className={`${css('cell')} ${css('code')}`} title={row.sha}>
        {row.shortSha}
      </div>
    </div>
  );
});

function AuthorCell({ row }: { readonly row: GraphRow }): ReactElement {
  return (
    <div role="gridcell" className={`${css('cell')} ${css('author')}`} title={row.authorEmail}>
      <span
        className={css('avatar')}
        aria-hidden="true"
        style={{ background: laneColor(row.lane) }}
      >
        {authorInitials(row.authorName)}
      </span>
      <span className={css('author-name')}>{row.authorName}</span>
    </div>
  );
}

/**
 * Fecha del autor en el formato corto del idioma (30/9/26), que cabe tambien en
 * la columna estrecha de VS Code; la fecha y la hora completas van en el title.
 * Si el motor mandara algo que no es una fecha, se ensena tal cual.
 */
function DateCell({ iso }: { readonly iso: string }): ReactElement {
  const locale = useLocale();
  const time = Date.parse(iso);
  const valid = !Number.isNaN(time);
  return (
    <div role="gridcell" className={`${css('cell')} ${css('code')}`}>
      <time dateTime={iso} title={valid ? formatDate(time, locale, { style: 'dateTime' }) : iso}>
        {valid ? formatDate(time, locale, { style: 'short' }) : iso}
      </time>
    </div>
  );
}
