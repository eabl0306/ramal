import { DEFAULT_THEME, type Language, type Theme } from './preferences';

/**
 * Idioma y tema efectivos de una pagina. La landing se prerenderiza en los dos
 * idiomas y su idioma sale de la URL (ADR 0015): `/` en castellano y `/en/` en
 * ingles. El resto de rutas son SPA y su idioma sale de la preferencia guardada.
 */

const LANDING_PATHS: Readonly<Record<Language, string>> = { es: '/', en: '/en/' };

/** Las rutas que se prerenderizan al construir, una por idioma. */
export const PRERENDERED_PATHS: readonly string[] = Object.values(LANDING_PATHS);

export function landingPath(language: Language): string {
  return LANDING_PATHS[language];
}

/** El idioma de una URL de la landing, o null si la ruta no es la landing. */
export function landingLanguage(pathname: string): Language | null {
  if (pathname === '' || pathname === '/') return 'es';
  if (pathname === '/en' || pathname === '/en/') return 'en';
  return null;
}

/** La URL manda en la landing; fuera de ella, lo que el visitante eligio o su navegador. */
export function pageLanguage(pathname: string, preferred: Language): Language {
  return landingLanguage(pathname) ?? preferred;
}

/** Lo elegido en el conmutador gana al sistema, y sin ninguno de los dos, el del diseno. */
export function effectiveTheme(stored: Theme | null, system: Theme | null): Theme {
  return stored ?? system ?? DEFAULT_THEME;
}
