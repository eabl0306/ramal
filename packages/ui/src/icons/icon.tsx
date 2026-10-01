import type { ReactElement, ReactNode } from 'react';

import type { IconName } from './icon-names';
import { type IconSet, IconSetContext, useIconSet } from './icon-set';

export interface IconProviderProps {
  readonly icons: IconSet;
  readonly children: ReactNode;
}

export function IconProvider({ icons, children }: IconProviderProps): ReactElement {
  return <IconSetContext value={icons}>{children}</IconSetContext>;
}

export interface IconProps {
  readonly name: IconName;
  /** Lado en px. Los glifos del diseno van de 12 a 17 px; 16 es el comun. */
  readonly size?: number;
  /**
   * Nombre accesible. Sin el, el icono es decorativo y queda oculto a los
   * lectores de pantalla: lo normal cuando va junto a un texto.
   */
  readonly label?: string;
  readonly className?: string;
}

/** Grosor de trazo de Lucide que se parece al peso de los glifos del prototipo. */
const STROKE_WIDTH = 1.75;

export function Icon({ name, size = 16, label, className }: IconProps): ReactElement {
  const set = useIconSet();
  const a11y =
    label === undefined
      ? { 'aria-hidden': true as const }
      : { role: 'img' as const, 'aria-label': label };
  if (set.kind === 'component') {
    const Glyph = set.icons[name];
    return <Glyph size={size} strokeWidth={STROKE_WIDTH} className={className} {...a11y} />;
  }
  const classes = ['codicon', set.icons[name], className].filter(Boolean).join(' ');
  return <span className={classes} style={{ fontSize: size }} {...a11y} />;
}
