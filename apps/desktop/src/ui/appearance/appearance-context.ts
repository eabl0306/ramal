import type { Locale } from '@git-rudder/i18n';
import { createContext, use } from 'react';

import type { ThemeName } from '../../application/ports/appearance-store';

/** Tema e idioma con los que se pinta la ventana. */
export interface Appearance {
  readonly theme: ThemeName;
  readonly locale: Locale;
}

export interface AppearanceValue extends Appearance {
  readonly setTheme: (theme: ThemeName) => void;
  readonly setLocale: (locale: Locale) => void;
}

export const AppearanceContext = createContext<AppearanceValue | null>(null);

export function useAppearance(): AppearanceValue {
  const value = use(AppearanceContext);
  if (value === null) {
    throw new Error('useAppearance necesita un <AppearanceProvider> por encima en el arbol.');
  }
  return value;
}

/**
 * Lleva la apariencia al documento: `data-theme` elige los valores de
 * `@git-rudder/ui/themes/git-rudder.css` y `lang` dice a lectores de pantalla y al
 * navegador en que idioma esta el texto.
 */
export function applyAppearance(root: HTMLElement, { theme, locale }: Appearance): void {
  root.setAttribute('data-theme', theme);
  root.setAttribute('lang', locale);
}
