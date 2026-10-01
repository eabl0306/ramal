import type { ReactElement, ReactNode } from 'react';
import { ToggleButton } from 'react-aria-components';

import type { Tone } from '../badge/badge';
import { moduleClasses } from '../module-classes';

import styles from './chip.module.css';

const css = moduleClasses(styles);

export interface ChipProps {
  readonly children: ReactNode;
  /** Color del punto delante del texto (estado de PR, color de workspace). */
  readonly tone?: Tone;
  /** Contador a la derecha, en monoespaciada. */
  readonly count?: number;
  readonly isSelected?: boolean;
  /**
   * Con onChange el chip es un filtro que se activa y desactiva (los chips de
   * estado de PR); sin el, es solo una etiqueta (el chip de workspace de un repo).
   */
  readonly onChange?: (isSelected: boolean) => void;
}

function ChipContent({ tone, count, children }: Omit<ChipProps, 'onChange'>): ReactElement {
  return (
    <>
      {tone === undefined ? null : <span className={css('dot')} data-tone={tone} />}
      <span className={css('label')}>{children}</span>
      {count === undefined ? null : <span className={css('count')}>{count}</span>}
    </>
  );
}

export function Chip({ isSelected = false, onChange, ...content }: ChipProps): ReactElement {
  if (onChange === undefined) {
    return (
      <span className={css('chip')}>
        <ChipContent {...content} />
      </span>
    );
  }
  return (
    <ToggleButton className={css('chip')} isSelected={isSelected} onChange={onChange}>
      <ChipContent {...content} />
    </ToggleButton>
  );
}
