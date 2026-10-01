import { readFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

/**
 * Un lector minimo de las hojas de temas y de componentes: bloques de primer
 * nivel y sus declaraciones. Basta para estas hojas, que no anidan reglas; no es
 * un parser de CSS general.
 */

export interface CssBlock {
  readonly selector: string;
  readonly declarations: ReadonlyMap<string, string>;
}

function stripComments(css: string): string {
  return css.replace(/\/\*[\s\S]*?\*\//gu, '');
}

/** Parte por ';' fuera de parentesis: un valor puede llevar comas y puntos y coma no. */
function splitDeclarations(body: string): string[] {
  const parts: string[] = [];
  let depth = 0;
  let current = '';
  for (const char of body) {
    if (char === '(') depth += 1;
    if (char === ')') depth -= 1;
    if (char === ';' && depth === 0) {
      parts.push(current);
      current = '';
      continue;
    }
    current += char;
  }
  parts.push(current);
  return parts;
}

/** Por defecto solo las propiedades personalizadas, que es lo que define un tema. */
function parseDeclarations(body: string, onlyCustom = true): Map<string, string> {
  const declarations = new Map<string, string>();
  for (const part of splitDeclarations(body)) {
    const colon = part.indexOf(':');
    if (colon === -1) continue;
    const name = part.slice(0, colon).trim();
    if (onlyCustom && !name.startsWith('--')) continue;
    const value = part.slice(colon + 1).replace(/\s+/gu, ' ');
    declarations.set(name, value.trim());
  }
  return declarations;
}

/** Ruta construida a mano: Vite reescribe `new URL(..., import.meta.url)` como un asset. */
export function readCss(relativeToThemes: string): string {
  const here = path.dirname(fileURLToPath(import.meta.url));
  return readFileSync(path.join(here, '..', 'themes', relativeToThemes), 'utf8');
}

/** Una hoja de src/, por ejemplo 'components/button/button.module.css'. */
export function readSourceCss(relativeToSrc: string): string {
  const here = path.dirname(fileURLToPath(import.meta.url));
  return readFileSync(path.join(here, '..', relativeToSrc), 'utf8');
}

/**
 * Todas las declaraciones de las reglas cuyo selector es exactamente el dado,
 * en el orden de la hoja (la ultima gana, como en el navegador).
 */
export function ruleOf(css: string, selector: string): ReadonlyMap<string, string> {
  const merged = new Map<string, string>();
  for (const match of stripComments(css).matchAll(/([^{}]+)\{([^{}]*)\}/gu)) {
    const selectors = (match[1] ?? '').split(',').map((part) => part.replace(/\s+/gu, ' ').trim());
    if (!selectors.includes(selector)) continue;
    for (const [name, value] of parseDeclarations(match[2] ?? '', false)) merged.set(name, value);
  }
  if (merged.size === 0) throw new Error(`no hay regla "${selector}"`);
  return merged;
}

export function parseBlocks(css: string): CssBlock[] {
  const blocks: CssBlock[] = [];
  for (const match of stripComments(css).matchAll(/([^{}]+)\{([^{}]*)\}/gu)) {
    const selector = (match[1] ?? '').replace(/\s+/gu, ' ').trim();
    blocks.push({ selector, declarations: parseDeclarations(match[2] ?? '') });
  }
  return blocks;
}

/** El primer bloque cuyo selector contiene el texto dado. */
export function blockWith(blocks: readonly CssBlock[], selectorPart: string): CssBlock {
  const block = blocks.find((candidate) => candidate.selector.includes(selectorPart));
  if (block === undefined) throw new Error(`no hay bloque con "${selectorPart}"`);
  return block;
}

export function allDeclarations(blocks: readonly CssBlock[]): Set<string> {
  return new Set(blocks.flatMap((block) => [...block.declarations.keys()]));
}

/** oklch(18.5% 0.008 230deg / 14%) a [0.185, 0.008, 230, 0.14]; sin alfa, alfa 1. */
export function oklchNumbers(value: string): number[] {
  const match = /^oklch\(([^)]*)\)$/u.exec(value.trim());
  if (match === null) throw new Error(`no es un oklch: ${value}`);
  const [channels = '', alpha = '1'] = (match[1] ?? '').split('/');
  const toNumber = (raw: string): number =>
    raw.endsWith('%') ? Number(raw.slice(0, -1)) / 100 : Number(raw.replace(/deg$/u, ''));
  return [...channels.trim().split(/\s+/u), alpha.trim()].map(toNumber);
}
