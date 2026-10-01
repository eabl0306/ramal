import { describe, expect, it } from 'vitest';

import { formatDate, formatRelativeTime, formatShortcut, formatTimeZone } from './format';
import { isLocale, resolveLocale } from './locale';

describe('formatRelativeTime', () => {
  it('dice ahora y ayer en vez de "hace 0 min" y "hace 1 d"', () => {
    expect(formatRelativeTime(0, 'second', 'es')).toBe('ahora');
    expect(formatRelativeTime(0, 'second', 'en')).toBe('now');
    expect(formatRelativeTime(-1, 'day', 'es')).toBe('ayer');
    expect(formatRelativeTime(-1, 'day', 'en')).toBe('yesterday');
  });

  it('escribe minutos, semanas y meses en cada idioma', () => {
    expect(formatRelativeTime(-5, 'minute', 'es')).toBe('hace 5 minutos');
    expect(formatRelativeTime(-2, 'week', 'en')).toBe('2 weeks ago');
    expect(formatRelativeTime(-3, 'month', 'es')).toBe('hace 3 meses');
  });
});

describe('formatDate', () => {
  const instant = Date.UTC(2026, 8, 30, 10, 5);

  it('formatea la fecha con el idioma de la interfaz', () => {
    expect(formatDate(instant, 'es', { timeZone: 'UTC' })).toBe('30 sept 2026');
    expect(formatDate(instant, 'en', { timeZone: 'UTC' })).toBe('Sep 30, 2026');
  });

  it('admite fecha con hora y solo hora', () => {
    expect(formatDate(instant, 'es', { style: 'dateTime', timeZone: 'UTC' })).toBe(
      '30 sept 2026, 10:05',
    );
    expect(formatDate(new Date(instant), 'es', { style: 'time', timeZone: 'UTC' })).toBe('10:05');
    expect(formatDate(instant, 'en')).toMatch(/2026/u);
  });
});

describe('formatTimeZone', () => {
  it('cambia todos los guiones bajos, no solo el primero', () => {
    expect(formatTimeZone('America/North_Dakota/New_Salem')).toBe('America/North Dakota/New Salem');
  });
});

describe('formatShortcut', () => {
  it('usa simbolos en macOS y nombres en el resto', () => {
    expect(formatShortcut('mod+enter', 'mac')).toBe('⌘↵');
    expect(formatShortcut('mod+enter', 'other')).toBe('Ctrl+Enter');
    expect(formatShortcut('shift + mod + g', 'mac')).toBe('⇧⌘G');
    expect(formatShortcut('shift+mod+g', 'other')).toBe('Shift+Ctrl+G');
  });
});

describe('resolveLocale', () => {
  it('castellano solo si la subetiqueta principal es es', () => {
    for (const tag of ['es', 'es-ES', 'es-419', 'ES', 'es_ES.UTF-8']) {
      expect(resolveLocale(tag)).toBe('es');
    }
  });

  it('cualquier otro idioma o nada da ingles', () => {
    for (const tag of ['en', 'en-US', 'fr', 'est', 'pt-BR', '', '  ', null, undefined]) {
      expect(resolveLocale(tag)).toBe('en');
    }
  });

  it('reconoce los idiomas soportados', () => {
    expect(isLocale('es')).toBe(true);
    expect(isLocale('fr')).toBe(false);
    expect(isLocale(undefined)).toBe(false);
  });
});
