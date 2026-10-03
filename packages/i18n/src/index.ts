// API publica de @git-rudder/i18n: idiomas, traductor tipado y formatos. El enlace
// con React (I18nProvider, useT) vive en @git-rudder/ui para que este paquete siga
// sin dependencias.

export { FALLBACK_LOCALE, LOCALES, type Locale, isLocale, resolveLocale } from './locale';
export { type Catalog, NAMESPACES } from './catalog';
export type {
  ParamName,
  PlainKey,
  Translate,
  TranslateArgs,
  TranslationKey,
  TranslationParams,
} from './keys';
export { createTranslator } from './translator';
export {
  type DateFormatOptions,
  type DateStyle,
  type RelativeTimeUnit,
  type ShortcutPlatform,
  formatDate,
  formatRelativeTime,
  formatShortcut,
  formatTimeZone,
} from './format';
