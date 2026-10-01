import type { ReactElement, ReactNode } from 'react';
import { Button as AriaButton, type ButtonProps as AriaButtonProps } from 'react-aria-components';

import { Icon } from '../../icons/icon';
import type { IconName } from '../../icons/icon-names';
import { moduleClasses } from '../module-classes';

import styles from './button.module.css';

const css = moduleClasses(styles);

/**
 * `outline` es el boton de proveedor de la tarjeta de entrar de la web: fondo de
 * la pagina y borde que se enciende con el acento al pasar, sin cambiar el fondo.
 */
export type ButtonVariant = 'primary' | 'secondary' | 'ghost' | 'danger' | 'outline';

/**
 * Las alturas del diseno: 30 (pequeno), 34, 36 (el comun), 38 (bloque) y, solo en
 * la web, 42 (descargas y proveedores), 44 (enviar en entrar y crear cuenta) y 48
 * (portada).
 */
export type ButtonSize = 30 | 34 | 36 | 38 | 42 | 44 | 48;

export interface ButtonProps extends Omit<AriaButtonProps, 'className' | 'style' | 'children'> {
  readonly variant?: ButtonVariant;
  readonly size?: ButtonSize;
  /** Ocupa todo el ancho (Commit, Completar merge). */
  readonly block?: boolean;
  /** Icono delante del texto, en lugar de un glifo dentro del texto. */
  readonly icon?: IconName;
  readonly children: ReactNode;
}

const ICON_SIZE: Readonly<Record<ButtonSize, number>> = {
  30: 14,
  34: 15,
  36: 16,
  38: 16,
  42: 17,
  44: 17,
  48: 18,
};

/**
 * Boton de texto sobre React Aria: teclado, foco visible y estados de pulsado
 * los pone la biblioteca; el aspecto sale de los tokens. La variante y el tamano
 * van en atributos data-* para que el CSS los lea igual que los estados de React
 * Aria (data-hovered, data-pressed, data-disabled).
 */
export function Button({
  variant = 'secondary',
  size = 36,
  block = false,
  icon,
  children,
  ...rest
}: ButtonProps): ReactElement {
  return (
    <AriaButton
      {...rest}
      className={css('button')}
      data-variant={variant}
      data-size={size}
      data-block={block ? 'true' : undefined}
    >
      {icon === undefined ? null : <Icon name={icon} size={ICON_SIZE[size]} />}
      {children}
    </AriaButton>
  );
}
