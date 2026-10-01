import { catalogFor } from './catalog';
import type { Locale } from './locale';

/**
 * Los diccionarios aplanados a "espacio.clave.sub" una sola vez por idioma, para
 * que cada traduccion sea una busqueda en un Map y no un recorrido del arbol.
 */

interface Tree {
  readonly [key: string]: string | Tree;
}

function flatten(tree: Tree, prefix: string, out: Map<string, string>): Map<string, string> {
  for (const [key, value] of Object.entries(tree)) {
    const full = prefix === '' ? key : `${prefix}.${key}`;
    if (typeof value === 'string') out.set(full, value);
    else flatten(value, full, out);
  }
  return out;
}

export function flattenTree(tree: Tree): Map<string, string> {
  return flatten(tree, '', new Map());
}

const cache = new Map<Locale, ReadonlyMap<string, string>>();

export function messagesFor(locale: Locale): ReadonlyMap<string, string> {
  const cached = cache.get(locale);
  if (cached !== undefined) return cached;
  const messages = flattenTree(catalogFor(locale));
  cache.set(locale, messages);
  return messages;
}
