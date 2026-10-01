import { type Locale, createTranslator } from '@ramal/i18n';
import { type ReactElement, type ReactNode, useMemo } from 'react';
import { I18nProvider as AriaI18nProvider } from 'react-aria-components';

import { I18nContext, type I18nValue } from './i18n-context';

/**
 * React Aria trae sus propios textos de accesibilidad y formatos; se le pasa el
 * mismo idioma para que la interfaz no quede mitad en un idioma y mitad en otro.
 */
const ARIA_LOCALES: Readonly<Record<Locale, string>> = { es: 'es-ES', en: 'en-US' };

export interface I18nProviderProps {
  readonly locale: Locale;
  readonly children: ReactNode;
}

/** Lo pone cada host en su composition root con el idioma que le toca. */
export function I18nProvider({ locale, children }: I18nProviderProps): ReactElement {
  const value = useMemo<I18nValue>(() => ({ locale, t: createTranslator(locale) }), [locale]);
  return (
    <I18nContext value={value}>
      <AriaI18nProvider locale={ARIA_LOCALES[locale]}>{children}</AriaI18nProvider>
    </I18nContext>
  );
}
