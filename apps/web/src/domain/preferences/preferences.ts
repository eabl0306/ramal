/**
 * Preferencias de presentacion que la web recuerda en el navegador: idioma y
 * tema. Los idiomas son los mismos que los de @ramal/i18n, que la capa de
 * aplicacion no puede importar (architecture.md 5.2), por eso se nombran aqui.
 */

export const LANGUAGES = ['es', 'en'] as const;
export type Language = (typeof LANGUAGES)[number];

export const THEMES = ['dark', 'light'] as const;
export type Theme = (typeof THEMES)[number];

/** El oscuro es el tema por defecto del diseno. */
export const DEFAULT_THEME: Theme = 'dark';

/** La landing en castellano es la raiz del sitio: sin otra senal, se habla castellano. */
export const DEFAULT_LANGUAGE: Language = 'es';

export function isLanguage(value: unknown): value is Language {
  return value === 'es' || value === 'en';
}

export function isTheme(value: unknown): value is Theme {
  return value === 'dark' || value === 'light';
}

/** Lo guardado; `null` es que el visitante todavia no ha elegido. */
export interface StoredPreferences {
  readonly language: Language | null;
  readonly theme: Theme | null;
}

export const NO_PREFERENCES: StoredPreferences = { language: null, theme: null };
