import type { ReactElement } from 'react';
import { Button as AriaButton, type ButtonProps as AriaButtonProps } from 'react-aria-components';

import { Icon } from '../../icons/icon';
import type { IconName } from '../../icons/icon-names';
import { moduleClasses } from '../module-classes';

import styles from './icon-button.module.css';

const css = moduleClasses(styles);

export type IconButtonVariant = 'ghost' | 'bordered';

/** 22 (cerrar pestana), 24, 28 (detalle), 30 (el comun, ⋯) y 36 (cabecera de workspace). */
export type IconButtonSize = 22 | 24 | 28 | 30 | 36;

export interface IconButtonProps extends Omit<
  AriaButtonProps,
  'className' | 'style' | 'children' | 'aria-label'
> {
  readonly icon: IconName;
  /** Obligatorio: un boton sin texto necesita nombre accesible. Viene de @ramal/i18n. */
  readonly label: string;
  readonly variant?: IconButtonVariant;
  readonly size?: IconButtonSize;
}

const GLYPH_SIZE: Readonly<Record<IconButtonSize, number>> = {
  22: 14,
  24: 14,
  28: 15,
  30: 16,
  36: 17,
};

export function IconButton({
  icon,
  label,
  variant = 'ghost',
  size = 30,
  ...rest
}: IconButtonProps): ReactElement {
  return (
    <AriaButton
      {...rest}
      aria-label={label}
      className={css('button')}
      data-variant={variant}
      data-size={size}
    >
      <Icon name={icon} size={GLYPH_SIZE[size]} />
    </AriaButton>
  );
}
