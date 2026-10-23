import { readdirSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { describe, expect, it } from 'vitest';

import { CATALOGS, NAMESPACES } from '../src/catalog';
import { PARAMS } from '../src/generated/params';
import { placeholdersOf } from '../src/interpolate';
import { PLURAL_SUFFIXES } from '../src/keys';
import { LOCALES } from '../src/locale';
import { flattenTree } from '../src/messages';

/**
 * La red de seguridad de los diccionarios: lo que el tipado de un JSON no puede
 * ver (marcadores y plurales) y lo que el prototipo hacia mal (glifos, atajos de
 * Mac y datos simulados dentro de los textos, gaps B-43 y B-46).
 */

// Ruta construida a mano: Vite reescribe `new URL(..., import.meta.url)` como un asset.
const LOCALES_DIR = path.join(path.dirname(fileURLToPath(import.meta.url)), '..', 'src', 'locales');
const es = flattenTree(CATALOGS.es);
const en = flattenTree(CATALOGS.en);
const PLURAL = new RegExp(`_(${PLURAL_SUFFIXES.join('|')})$`, 'u');

function sortedKeys(messages: ReadonlyMap<string, string>): string[] {
  return [...messages.keys()].sort();
}

function pluralBases(messages: ReadonlyMap<string, string>): Set<string> {
  return new Set(
    [...messages.keys()].filter((key) => PLURAL.test(key)).map((key) => key.replace(PLURAL, '')),
  );
}

/** La misma tabla que escribe scripts/generate-params.mjs. */
function expectedParams(): Record<string, string[]> {
  const table = new Map<string, Set<string>>();
  for (const [key, text] of es) {
    const base = key.replace(PLURAL, '');
    const names = table.get(base) ?? new Set<string>();
    if (base !== key) names.add('count');
    for (const name of placeholdersOf(text)) names.add(name);
    table.set(base, names);
  }
  const entries = [...table]
    .filter(([, names]) => names.size > 0)
    .map(([key, names]): [string, string[]] => [key, [...names].sort()]);
  return Object.fromEntries(entries);
}

describe('paridad de los diccionarios', () => {
  it('castellano e ingles tienen exactamente las mismas claves', () => {
    expect(sortedKeys(en)).toEqual(sortedKeys(es));
    expect(es.size).toBeGreaterThan(600);
  });

  it('cada clave lleva los mismos marcadores en los dos idiomas', () => {
    const mismatches = [...es]
      .filter(([key, text]) => {
        const other = en.get(key) ?? '';
        return placeholdersOf(text).sort().join() !== placeholdersOf(other).sort().join();
      })
      .map(([key]) => key);
    expect(mismatches).toEqual([]);
  });

  it('cada plural tiene _one y _other en los dos idiomas', () => {
    for (const messages of [es, en]) {
      for (const base of pluralBases(messages)) {
        expect(messages.has(`${base}_one`), `${base}_one`).toBe(true);
        expect(messages.has(`${base}_other`), `${base}_other`).toBe(true);
        expect(messages.has(base), `${base} sin sufijo`).toBe(false);
      }
    }
  });

  it('la forma _other de un plural ensena el recuento', () => {
    for (const base of pluralBases(es)) {
      expect(placeholdersOf(es.get(`${base}_other`) ?? ''), base).toContain('count');
    }
  });

  it('la tabla generada de parametros esta al dia (corepack pnpm --filter @git-rudder/i18n generate)', () => {
    const generated = Object.fromEntries(
      Object.entries(PARAMS).map(([key, names]) => [key, [...names]]),
    );
    expect(generated).toEqual(expectedParams());
  });
});

describe('contenido de los textos', () => {
  it('ningun texto esta vacio', () => {
    const empty = [...es, ...en].filter(([, text]) => text.trim() === '').map(([key]) => key);
    expect(empty).toEqual([]);
  });

  it('ningun texto lleva glifos ni atajos de teclado escritos', () => {
    const glyphs = /[✓×●✦›⌘↵⌥⇧⌃]|Ctrl\+|Cmd\+/u;
    const offenders = [...es, ...en].filter(([, text]) => glyphs.test(text)).map(([key]) => key);
    expect(offenders).toEqual([]);
  });

  it('ningun texto de producto dice que algo es simulado', () => {
    const offenders = [...es, ...en]
      .filter(([, text]) => /simulad[oa]|simulated/iu.test(text))
      .map(([key]) => key);
    expect(offenders).toEqual([]);
  });

  it('los marcadores tienen nombres descriptivos, no de una letra', () => {
    const short = [...es, ...en]
      .filter(([, text]) => placeholdersOf(text).some((name) => name.length < 2))
      .map(([key]) => key);
    expect(short).toEqual([]);
  });
});

describe('ficheros de los diccionarios', () => {
  it('cada JSON de cada idioma esta conectado al catalogo', () => {
    for (const locale of ['es', 'en']) {
      const files = readdirSync(path.join(LOCALES_DIR, locale))
        .filter((name) => name.endsWith('.json'))
        .map((name) => name.replace(/\.json$/u, ''))
        .sort();
      expect(files).toEqual([...NAMESPACES].sort());
    }
  });

  it('solo se ofrecen los idiomas que estan traducidos', () => {
    // El prototipo ofrecia frances sin traduccion (gaps B-47). La lista de idiomas
    // que ensena un selector sale de common.language, asi que tiene que tener una
    // entrada por idioma de LOCALES y ninguna mas.
    const offered = [...es.keys()]
      .filter((key) => key.startsWith('common.language.'))
      .map((key) => key.slice('common.language.'.length))
      .sort();
    expect(offered).toEqual([...LOCALES].sort());
  });

  it('los espacios de nombres son los de la arquitectura', () => {
    expect([...NAMESPACES].sort()).toEqual(
      [
        'common',
        'repos',
        'graph',
        'staging',
        'diff',
        'conflict',
        'rebase',
        'menu',
        'dialog',
        'toast',
        'settings',
        'ai',
        'team',
        'errors',
        'vscode',
        'web',
      ].sort(),
    );
  });
});
