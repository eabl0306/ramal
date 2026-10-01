import type { Locale } from './locale';

/**
 * Formatos que dependen del idioma: tiempo relativo, fechas, zonas horarias y
 * atajos de teclado. Sustituyen al `ago()` del prototipo, que tenia los textos
 * escritos a mano y fuera del diccionario (gaps B-48).
 */

export type RelativeTimeUnit = 'second' | 'minute' | 'hour' | 'day' | 'week' | 'month' | 'year';

/**
 * El tramo lo decide relativeTimeBucket de @ramal/domain; aqui solo se escribe.
 * `numeric: 'auto'` da "ahora" para 0 segundos y "ayer" para -1 dia.
 */
export function formatRelativeTime(value: number, unit: RelativeTimeUnit, locale: Locale): string {
  return new Intl.RelativeTimeFormat(locale, { numeric: 'auto' }).format(value, unit);
}

/** `short` es la fecha numerica corta (30/9/26) de las columnas estrechas, como la del grafo. */
export type DateStyle = 'date' | 'short' | 'dateTime' | 'time';

const DATE_OPTIONS: Readonly<Record<DateStyle, Intl.DateTimeFormatOptions>> = {
  date: { dateStyle: 'medium' },
  short: { dateStyle: 'short' },
  dateTime: { dateStyle: 'medium', timeStyle: 'short' },
  time: { timeStyle: 'short' },
};

export interface DateFormatOptions {
  readonly style?: DateStyle;
  /** Zona IANA; sin ella, la del sistema. Los tests la fijan para no depender de la maquina. */
  readonly timeZone?: string;
}

export function formatDate(
  date: Date | number,
  locale: Locale,
  options: DateFormatOptions = {},
): string {
  const base = DATE_OPTIONS[options.style ?? 'date'];
  const zone = options.timeZone === undefined ? {} : { timeZone: options.timeZone };
  return new Intl.DateTimeFormat(locale, { ...base, ...zone }).format(date);
}

/** "America/Buenos_Aires" a "America/Buenos Aires", con todos los guiones bajos (gaps B-49). */
export function formatTimeZone(zone: string): string {
  return zone.replaceAll('_', ' ');
}

export type ShortcutPlatform = 'mac' | 'other';

/** Simbolo en macOS y nombre en Windows y Linux. `mod` es Cmd en macOS y Ctrl en el resto. */
const KEYS: Readonly<Record<string, readonly [mac: string, other: string]>> = {
  mod: ['⌘', 'Ctrl'],
  ctrl: ['⌃', 'Ctrl'],
  alt: ['⌥', 'Alt'],
  shift: ['⇧', 'Shift'],
  enter: ['↵', 'Enter'],
  escape: ['⎋', 'Esc'],
  backspace: ['⌫', 'Backspace'],
  delete: ['⌦', 'Del'],
  tab: ['⇥', 'Tab'],
  space: ['␣', 'Space'],
  up: ['↑', '↑'],
  down: ['↓', '↓'],
  left: ['←', '←'],
  right: ['→', '→'],
};

/**
 * `formatShortcut("mod+enter", "mac")` da "⌘↵" y en Windows "Ctrl+Enter". Los
 * textos reciben el atajo como parametro en vez de llevarlo escrito (gaps B-43).
 */
export function formatShortcut(shortcut: string, platform: ShortcutPlatform): string {
  const index = platform === 'mac' ? 0 : 1;
  const parts = shortcut
    .split('+')
    .map((part) => part.trim().toLowerCase())
    .filter((part) => part !== '')
    .map((part) => KEYS[part]?.[index] ?? part.toUpperCase());
  return parts.join(platform === 'mac' ? '' : '+');
}
