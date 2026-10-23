import type { RefLabel } from '@git-rudder/bridge';
import { Icon, useT } from '@git-rudder/ui';
import type { ReactElement } from 'react';

import { refColor, refIcon, splitRefs } from '../graph/refs';

import { cssVars, moduleClasses } from './css';
import gridStyles from './graph-grid.module.css';
import styles from './ref-pills.module.css';

const css = moduleClasses(styles);
const grid = moduleClasses(gridStyles);

export interface RefPillsProps {
  readonly refs: readonly RefLabel[];
  /** Carril del commit: da el color de las pildoras de rama. */
  readonly lane: number;
}

/**
 * Celda de refs: como mucho dos pildoras y "+n" con el resto. Cada pildora lleva
 * el tipo de ref en texto oculto para los lectores de pantalla, porque el icono
 * es decorativo; "+n" les lee en su lugar cuantas refs esconde y cuales.
 */
export function RefPills({ refs, lane }: RefPillsProps): ReactElement {
  const t = useT();
  const { shown, hidden } = splitRefs(refs);
  const hiddenNames = hidden.map((ref) => ref.name).join(', ');
  return (
    <div role="gridcell" className={css('refs')}>
      {shown.map((ref) => (
        <span
          key={ref.fullName}
          className={css('ref')}
          data-head={ref.isHead}
          data-kind={ref.kind}
          title={ref.fullName}
          style={cssVars({ '--rv-ref-color': refColor(ref, lane) })}
        >
          <Icon name={refIcon(ref)} size={12} className={css('ref-icon')} />
          <span className={grid('visually-hidden')}>{t(kindKey(ref))}</span>
          <span className={css('ref-name')}>{ref.name}</span>
        </span>
      ))}
      {hidden.length > 0 ? (
        <span className={css('more')} title={hiddenNames}>
          <span aria-hidden="true">{t('graph.refs.more', { count: hidden.length })}</span>
          <span className={grid('visually-hidden')}>
            {t('graph.refs.moreLabel', { count: hidden.length, names: hiddenNames })}
          </span>
        </span>
      ) : null}
    </div>
  );
}

function kindKey(ref: RefLabel): `graph.refs.kind.${RefLabel['kind'] | 'head'}` {
  return ref.isHead ? 'graph.refs.kind.head' : `graph.refs.kind.${ref.kind}`;
}
