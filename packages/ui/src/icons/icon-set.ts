import { type ComponentType, createContext, use } from 'react';

import type { IconName } from './icon-names';

/** Las props que <Icon> pasa a un icono hecho componente (Lucide). */
export interface IconGlyphProps {
  readonly size?: number;
  readonly strokeWidth?: number;
  readonly className?: string | undefined;
  readonly 'aria-hidden'?: boolean;
  readonly role?: string;
  readonly 'aria-label'?: string;
}

/**
 * Un juego de iconos: componentes (Lucide, escritorio y web) o clases CSS
 * (Codicons, VS Code). El host elige uno en su composition root con
 * <IconProvider>; los componentes solo piden <Icon name="branch" />.
 */
export type IconSet =
  | {
      readonly kind: 'component';
      readonly icons: Readonly<Record<IconName, ComponentType<IconGlyphProps>>>;
    }
  | { readonly kind: 'class'; readonly icons: Readonly<Record<IconName, string>> };

export const IconSetContext = createContext<IconSet | null>(null);

export function useIconSet(): IconSet {
  const set = use(IconSetContext);
  if (set === null) {
    throw new Error('<Icon> necesita un <IconProvider> por encima en el arbol.');
  }
  return set;
}
