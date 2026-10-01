import type { Locale, Translate } from '@ramal/i18n';
import { createContext, use } from 'react';

/**
 * El enlace de @ramal/i18n con React. Vive en @ramal/ui, y no en @ramal/i18n,
 * para que el paquete de textos siga sin dependencias y lo pueda usar el
 * extension host de VS Code.
 */
export interface I18nValue {
  readonly locale: Locale;
  readonly t: Translate;
}

export const I18nContext = createContext<I18nValue | null>(null);

function useI18n(): I18nValue {
  const value = use(I18nContext);
  if (value === null) {
    throw new Error('useT necesita un <I18nProvider> por encima en el arbol.');
  }
  return value;
}

/** El traductor del idioma actual: `const t = useT(); t('common.save')`. */
export function useT(): Translate {
  return useI18n().t;
}

export function useLocale(): Locale {
  return useI18n().locale;
}
