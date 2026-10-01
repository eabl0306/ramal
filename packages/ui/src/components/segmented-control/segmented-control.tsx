import type { ReactElement } from 'react';
import { type Key, ToggleButton, ToggleButtonGroup } from 'react-aria-components';

import { Icon } from '../../icons/icon';
import type { IconName } from '../../icons/icon-names';
import { moduleClasses } from '../module-classes';

import styles from './segmented-control.module.css';

const css = moduleClasses(styles);

export interface SegmentOption<T extends string> {
  readonly id: T;
  /** Texto visible; con `icon` y sin texto, pasa a ser el nombre accesible. */
  readonly label: string;
  readonly icon?: IconName;
  /** Solo el icono, sin texto visible (el conmutador de tema de la barra). */
  readonly iconOnly?: boolean;
}

/**
 * Las cuatro variantes del inventario 2.8: `bar` (ES/EN y tema de la barra de
 * titulo), `row` (fila de ajustes), `field` (campo de un modal, a todo el ancho)
 * y `surface` (lado a lado o unificado del diff).
 */
export type SegmentedVariant = 'bar' | 'row' | 'field' | 'surface';

export interface SegmentedControlProps<T extends string> {
  readonly options: readonly SegmentOption<T>[];
  readonly value: T;
  readonly onChange: (value: T) => void;
  /** Nombre accesible del grupo. */
  readonly label: string;
  readonly variant?: SegmentedVariant;
}

/** Un solo valor elegido, nunca ninguno, con flechas de teclado de React Aria. */
export function SegmentedControl<T extends string>({
  options,
  value,
  onChange,
  label,
  variant = 'row',
}: SegmentedControlProps<T>): ReactElement {
  const select = (keys: Set<Key>): void => {
    const option = options.find((candidate) => keys.has(candidate.id));
    if (option !== undefined) onChange(option.id);
  };
  return (
    <ToggleButtonGroup
      className={css('group')}
      data-variant={variant}
      aria-label={label}
      selectionMode="single"
      disallowEmptySelection
      selectedKeys={[value]}
      onSelectionChange={select}
    >
      {options.map((option) => (
        <ToggleButton
          key={option.id}
          id={option.id}
          className={css('segment')}
          {...(option.iconOnly === true ? { 'aria-label': option.label } : {})}
        >
          {option.icon === undefined ? null : <Icon name={option.icon} size={14} />}
          {option.iconOnly === true ? null : option.label}
        </ToggleButton>
      ))}
    </ToggleButtonGroup>
  );
}
