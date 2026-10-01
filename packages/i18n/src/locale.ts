/**
 * Idiomas de Ramal y la regla comun para elegirlos (ADR 0017): castellano si la
 * subetiqueta principal es `es`, ingles en cualquier otro caso. La usan el
 * extension host con `vscode.env.language`, el escritorio en el primer arranque
 * con el idioma del sistema y el ajuste de IA "igual que VS Code".
 */

export const LOCALES = ['es', 'en'] as const;

export type Locale = (typeof LOCALES)[number];

/** El ingles es el idioma de reserva, igual que en VS Code. */
export const FALLBACK_LOCALE: Locale = 'en';

export function isLocale(value: unknown): value is Locale {
  return value === 'es' || value === 'en';
}

/**
 * `es`, `es-ES`, `es-419` y la forma POSIX `es_ES.UTF-8` dan castellano; todo lo
 * demas, incluido un valor vacio, da ingles.
 */
export function resolveLocale(tag: string | null | undefined): Locale {
  const primary = (tag ?? '')
    .trim()
    .toLowerCase()
    .split(/[-_.@]/u)[0];
  return primary === 'es' ? 'es' : FALLBACK_LOCALE;
}
