import { describe, expect, it } from 'vitest';

import {
  PRERENDERED_PATHS,
  effectiveTheme,
  landingLanguage,
  landingPath,
  pageLanguage,
} from './page-preferences';
import { isLanguage, isTheme } from './preferences';

describe('rutas de la landing', () => {
  it('prerenderiza la raiz en castellano y /en/ en ingles', () => {
    expect(PRERENDERED_PATHS).toEqual(['/', '/en/']);
    expect(landingPath('es')).toBe('/');
    expect(landingPath('en')).toBe('/en/');
  });

  it('lee el idioma de la URL de la landing con o sin barra final', () => {
    expect(landingLanguage('/')).toBe('es');
    expect(landingLanguage('')).toBe('es');
    expect(landingLanguage('/en')).toBe('en');
    expect(landingLanguage('/en/')).toBe('en');
  });

  it('no inventa idioma para rutas que no son la landing', () => {
    expect(landingLanguage('/login')).toBeNull();
    expect(landingLanguage('/en/login')).toBeNull();
  });
});

describe('pageLanguage', () => {
  it('en la landing manda la URL aunque la preferencia diga otra cosa', () => {
    expect(pageLanguage('/en/', 'es')).toBe('en');
    expect(pageLanguage('/', 'en')).toBe('es');
  });

  it('fuera de la landing usa la preferencia', () => {
    expect(pageLanguage('/status', 'en')).toBe('en');
  });
});

describe('effectiveTheme', () => {
  it('lo elegido gana al sistema y el sistema al tema por defecto', () => {
    expect(effectiveTheme('light', 'dark')).toBe('light');
    expect(effectiveTheme(null, 'light')).toBe('light');
    expect(effectiveTheme(null, null)).toBe('dark');
  });
});

describe('guardas de tipo', () => {
  it('solo acepta los idiomas y temas conocidos', () => {
    expect(isLanguage('es')).toBe(true);
    expect(isLanguage('fr')).toBe(false);
    expect(isTheme('light')).toBe(true);
    expect(isTheme('sepia')).toBe(false);
  });
});
