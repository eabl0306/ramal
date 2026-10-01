import type { ReactElement, ReactNode } from 'react';
import { SwitchButton, SwitchField } from 'react-aria-components';

import { moduleClasses } from '../module-classes';

import styles from './toggle.module.css';

const css = moduleClasses(styles);

export interface ToggleProps {
  readonly isSelected: boolean;
  readonly onChange: (isSelected: boolean) => void;
  /** Texto visible a la izquierda. En las filas de ajustes el texto va aparte y se usa `label`. */
  readonly children?: ReactNode;
  /** Nombre accesible cuando no hay texto visible. */
  readonly label?: string;
  readonly isDisabled?: boolean;
}

/** Interruptor de 38x22 del diseno, con el desplazamiento del pomo en .15s. */
export function Toggle({
  isSelected,
  onChange,
  children,
  label,
  isDisabled = false,
}: ToggleProps): ReactElement {
  return (
    <SwitchField
      className={css('field')}
      isSelected={isSelected}
      onChange={onChange}
      isDisabled={isDisabled}
      {...(label === undefined ? {} : { 'aria-label': label })}
    >
      <SwitchButton className={css('toggle')}>
        {children === undefined ? null : <span className={css('label')}>{children}</span>}
        <span className={css('track')} aria-hidden="true">
          <span className={css('knob')} />
        </span>
      </SwitchButton>
    </SwitchField>
  );
}
