import { I18nProvider } from '@git-rudder/ui';
import { type ReactElement, type ReactNode, useEffect, useMemo, useState } from 'react';

import type { AppearanceStore } from '../../application/ports/appearance-store';

import {
  type Appearance,
  AppearanceContext,
  type AppearanceValue,
  applyAppearance,
} from './appearance-context';

export interface AppearanceProviderProps {
  /** La apariencia del arranque: la guardada o, la primera vez, la del sistema. */
  readonly initial: Appearance;
  /** Donde se guarda lo que el usuario elige. */
  readonly store: AppearanceStore;
  /** El elemento que recibe `data-theme` y `lang`; `<html>` en la app. */
  readonly root: HTMLElement;
  readonly children: ReactNode;
}

/**
 * Tema e idioma de la ventana. Pone `data-theme` y `lang` en el documento,
 * guarda cada cambio del usuario y monta el `I18nProvider` de @git-rudder/ui con el
 * idioma elegido, asi que todo lo de dentro se traduce al cambiarlo.
 */
export function AppearanceProvider({
  initial,
  store,
  root,
  children,
}: AppearanceProviderProps): ReactElement {
  const [appearance, setAppearance] = useState(initial);

  useEffect(() => {
    applyAppearance(root, appearance);
  }, [root, appearance]);

  const value = useMemo<AppearanceValue>(() => {
    const choose = (next: Appearance): void => {
      setAppearance(next);
      store.save(next);
    };
    return {
      ...appearance,
      setTheme: (theme) => {
        choose({ ...appearance, theme });
      },
      setLocale: (locale) => {
        choose({ ...appearance, locale });
      },
    };
  }, [appearance, store]);

  return (
    <AppearanceContext value={value}>
      <I18nProvider locale={appearance.locale}>{children}</I18nProvider>
    </AppearanceContext>
  );
}
