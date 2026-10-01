import { readFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

/**
 * Lectura minima de las hojas de src/view para los tests. jsdom no aplica los
 * CSS Modules, asi que lo que el diseno pide (bordes de alto contraste, tintes de
 * las pildoras) solo se puede comprobar leyendo la regla. No es un parser de CSS
 * general: basta para estas hojas, que no anidan reglas.
 */

/** Ruta construida a mano: Vite reescribe `new URL(..., import.meta.url)` como un asset. */
export function readViewCss(file: string): string {
  const here = path.dirname(fileURLToPath(import.meta.url));
  return readFileSync(path.join(here, '..', 'view', file), 'utf8');
}

/** Las declaraciones de las reglas cuyo selector (o uno de su lista) es exactamente el dado. */
export function ruleOf(css: string, selector: string): ReadonlyMap<string, string> {
  const merged = new Map<string, string>();
  const clean = css.replace(/\/\*[\s\S]*?\*\//gu, '');
  for (const match of clean.matchAll(/([^{}]+)\{([^{}]*)\}/gu)) {
    const selectors = (match[1] ?? '').split(',').map((part) => part.replace(/\s+/gu, ' ').trim());
    if (!selectors.includes(selector)) continue;
    for (const declaration of (match[2] ?? '').split(';')) {
      const colon = declaration.indexOf(':');
      if (colon === -1) continue;
      merged.set(declaration.slice(0, colon).trim(), declaration.slice(colon + 1).trim());
    }
  }
  if (merged.size === 0) throw new Error(`no hay regla "${selector}"`);
  return merged;
}
