import { describe, expect, it } from 'vitest';

import { CATALOGS } from '../src/catalog';
import { PLURAL_SUFFIXES } from '../src/keys';
import { flattenTree } from '../src/messages';

/**
 * Lo que la paridad no puede ver: que dos claves signifiquen lo mismo.
 *
 * El castellano es la fuente de verdad, asi que dos claves con el mismo texto
 * castellano tienen que llevar el mismo ingles. Cuando no coincide, una de las dos
 * traducciones esta mal: es el fallo que el analisis de los prototipos encontro
 * entre `D` y `W` ("Revoke" traducido como "Cerrar" en un sitio y "Desconectar" en
 * otro) y el que vuelve con `STATES`
 * ([ADR 0017](../../../docs/adr/0017-internacionalizacion-compartida.md)).
 *
 * Separar los espacios de nombres hace que las claves no choquen, pero no impide
 * eso: `common.cancel` y `web.common.cancel` conviven sin chocar y se separan el dia
 * que alguien cambia una. De ahi las reglas de aqui.
 */

const es = flattenTree(CATALOGS.es);
const en = flattenTree(CATALOGS.en);
const PLURAL = new RegExp(`_(${PLURAL_SUFFIXES.join('|')})$`, 'u');

/** Una excepcion: el conjunto exacto de claves que puede separarse, y por que. */
interface SharedText {
  readonly keys: readonly string[];
  readonly reason: string;
}

/**
 * Mismo castellano y distinto ingles solo donde el castellano tiene una palabra
 * para dos cosas que el ingles distingue.
 */
const SAME_ES: readonly SharedText[] = [
  {
    keys: ['common.username', 'settings.group.user'],
    reason:
      '"Usuario" nombra a la persona (el grupo de ajustes) y al campo del nombre de usuario; en ingles son "User" y "Username"',
  },
  {
    keys: ['settings.section.integrations', 'vscode.settings.section.platforms'],
    reason: 'el manifiesto de VS Code escribe las categorias de ajustes en Title Case',
  },
];

/**
 * Mismo ingles y distinto castellano es menos grave: el traductor busca por clave,
 * no por texto ingles, que es justo por lo que no se usa `vscode.l10n`. Aun asi se
 * declara, porque la mayoria de las veces es una traduccion descuidada.
 */
const SAME_EN: readonly SharedText[] = [
  {
    keys: ['menu.open', 'repos.open', 'repos.pr.state.open'],
    reason: '"Open" es el verbo del menu ("Abrir") y el estado de un pull request ("Abierto")',
  },
  {
    keys: ['repos.filter.allPlatforms', 'repos.filter.kind.all', 'repos.pr.state.all'],
    reason: 'concordancia de genero: "Todas" las plataformas y "Todos" los repositorios y estados',
  },
  {
    keys: ['staging.commitLabel', 'vscode.scm.commit', 'vscode.scm.commitButton'],
    reason:
      '"Commit" es el sustantivo (la etiqueta y el comando, que en castellano se queda en "Commit") y el verbo del boton del panel SCM, que es "Confirmar" (gaps B-47)',
  },
];

/**
 * Un numero sin sustantivo al lado no concuerda con nada, asi que no lleva plural.
 */
const COUNT_WITHOUT_PLURAL: Readonly<Record<string, string>> = {
  'graph.refs.more': 'la insignia "+3" del grafo es un numero suelto, sin sustantivo',
};

function signature(keys: readonly string[]): string {
  return [...keys].sort().join(', ');
}

/**
 * Los grupos de claves que comparten el texto en `source` y no en `target`, cada uno
 * como "texto: clave, clave" para que el fallo se lea sin abrir los diccionarios.
 */
function divergent(
  source: ReadonlyMap<string, string>,
  target: ReadonlyMap<string, string>,
): Map<string, string[]> {
  const groups = new Map<string, string[]>();
  for (const [key, text] of source) {
    groups.set(text, [...(groups.get(text) ?? []), key]);
  }
  const out = new Map<string, string[]>();
  for (const [text, keys] of groups) {
    if (keys.length < 2) continue;
    if (new Set(keys.map((key) => target.get(key))).size < 2) continue;
    out.set(text, keys);
  }
  return out;
}

function offenders(
  source: ReadonlyMap<string, string>,
  target: ReadonlyMap<string, string>,
  exceptions: readonly SharedText[],
): string[] {
  const allowed = new Set(exceptions.map((entry) => signature(entry.keys)));
  return [...divergent(source, target)]
    .filter(([, keys]) => !allowed.has(signature(keys)))
    .map(([text, keys]) => `${text}: ${signature(keys)}`);
}

describe('dos claves con el mismo texto', () => {
  it('con el mismo castellano llevan el mismo ingles', () => {
    expect(offenders(es, en, SAME_ES)).toEqual([]);
  });

  it('con el mismo ingles llevan el mismo castellano', () => {
    expect(offenders(en, es, SAME_EN)).toEqual([]);
  });

  it('cada excepcion existe, sigue haciendo falta y lleva su motivo', () => {
    for (const [exceptions, groups] of [
      [SAME_ES, divergent(es, en)],
      [SAME_EN, divergent(en, es)],
    ] as const) {
      const live = new Set([...groups.values()].map(signature));
      for (const entry of exceptions) {
        for (const key of entry.keys) expect(es.has(key), key).toBe(true);
        expect(entry.reason.trim(), signature(entry.keys)).not.toBe('');
        expect(live.has(signature(entry.keys)), `${signature(entry.keys)} ya no se separa`).toBe(
          true,
        );
      }
    }
  });
});

describe('common es el unico sitio de un texto compartido', () => {
  const pairOf = (key: string): string => `${es.get(key) ?? ''}\u0000${en.get(key) ?? ''}`;

  it('ninguna clave de otro espacio repite un texto de common', () => {
    const inCommon = new Map<string, string>();
    for (const key of es.keys()) {
      if (key.startsWith('common.')) inCommon.set(pairOf(key), key);
    }
    const echoes = [...es.keys()]
      .filter((key) => !key.startsWith('common.'))
      .map((key) => ({ key, original: inCommon.get(pairOf(key)) }))
      .filter((echo) => echo.original !== undefined)
      .map((echo) => `${echo.key} repite ${echo.original ?? ''}`);
    expect(echoes).toEqual([]);
  });
});

describe('los recuentos', () => {
  it('un texto con {count} tiene formas plurales', () => {
    const missing = [...es]
      .filter(([key, text]) => !PLURAL.test(key) && text.includes('{count}'))
      .map(([key]) => key)
      .filter((key) => !(key in COUNT_WITHOUT_PLURAL));
    expect(missing).toEqual([]);
  });

  it('cada clave sin plural declarada existe, sigue sin plural y lleva su motivo', () => {
    for (const [key, reason] of Object.entries(COUNT_WITHOUT_PLURAL)) {
      expect(es.has(key), key).toBe(true);
      expect(reason.trim(), key).not.toBe('');
      expect((es.get(key) ?? '').includes('{count}'), `${key} ya no lleva {count}`).toBe(true);
    }
  });
});
