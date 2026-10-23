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
    keys: ['web.auth.authorize.device', 'web.dashboard.nav.team'],
    reason:
      '"Equipo" es la maquina que pide autorizacion y el equipo de personas; en ingles son "Device" y "Team"',
  },
  {
    keys: ['vscode.scm.commitButton', 'web.auth.reauth.confirm'],
    reason:
      '"Confirmar" es el verbo generico de confirmar y el verbo de hacer un commit; en ingles son "Confirm" y "Commit"',
  },
  {
    keys: ['common.dismiss', 'vscode.confirm.discard.button'],
    reason:
      '"Descartar" es cerrar un aviso y tirar unos cambios; en ingles son "Dismiss" y "Discard"',
  },
  {
    keys: ['common.delete', 'vscode.confirm.dropStash.button'],
    reason: '"Eliminar" es borrar algo y tirar un stash, que en git es "Drop" y no "Delete"',
  },
  {
    keys: ['common.ok', 'web.team.invAccept'],
    reason:
      '"Aceptar" es el boton de un dialogo y aceptar una invitacion; en ingles son "OK" y "Accept"',
  },
  {
    keys: ['web.auth.deviceCode.reject', 'web.team.invDecline'],
    reason:
      '"Rechazar" es rechazar un codigo de equipo y declinar una invitacion; en ingles son "Reject" y "Decline"',
  },
  {
    keys: ['web.dashboard.dcIndex', 'web.footer.docs'],
    reason:
      '"Documentacion" es el titulo de la pagina y la etiqueta corta del pie; en ingles son "Documentation" y "Docs"',
  },
  {
    keys: ['web.billing.coSummary', 'web.dashboard.nav.overview'],
    reason:
      '"Resumen" es el resumen del pedido y la vista general del panel; en ingles son "Summary" y "Overview"',
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
    keys: ['graph.sidebar.local', 'repos.filter.kind.local', 'vscode.quickInput.local'],
    reason:
      'concordancia: "Local" es un repositorio y la seccion de refs, y "Locales" son las ramas del selector de VS Code; en ingles las tres son "Local"',
  },
  {
    keys: ['repos.filter.kind.remote', 'vscode.quickInput.remote'],
    reason:
      'concordancia: "Remoto" es un repositorio y "Remotas" son las ramas del selector de VS Code; en ingles las dos son "Remote"',
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

/**
 * VS Code escribe en Title Case los comandos, las vistas, los menus, los ajustes y
 * el recorrido de bienvenida: es su convencion y la lee su manifiesto. Asi que una
 * clave de `vscode` puede decir "Force Push (with Lease)…" donde la app dice
 * "Force push (with lease)…", y eso no es una inconsistencia. Lo que no puede es
 * decir otra cosa: si al pasar a minusculas los dos ingleses siguen sin coincidir,
 * la regla salta igual.
 */
function onlyTitleCaseInVsCode(
  keys: readonly string[],
  target: ReadonlyMap<string, string>,
): boolean {
  if (!keys.some((key) => key.startsWith('vscode.'))) return false;
  const folded = new Set(keys.map((key) => (target.get(key) ?? '').toLowerCase()));
  return folded.size === 1;
}

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
    .filter(([, keys]) => !onlyTitleCaseInVsCode(keys, target))
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

/**
 * `errors.engine` y `errors.api` no son dos grupos cualquiera: son las tablas que
 * traducen el `kind` del motor y el `code` de la API, y la interfaz las consulta con
 * ese valor ([ADR 0017](../../../docs/adr/0017-internacionalizacion-compartida.md)).
 * Una clave en camelCase ahi es un texto que nadie va a encontrar, y puede chocar
 * el dia que ese nombre sea un code de verdad.
 */
/**
 * Una palabra que el producto no usa en ningun sitio no puede colarse en un texto
 * nuevo. Hoy solo hay una: el producto dice "Sign out" y "Signed out" en todas
 * partes y no dice "Log out" en ninguna, asi que el verbo de entrar es "sign in" y
 * no "log in". El castellano no tiene el problema: siempre es "iniciar sesion".
 */
const BANNED_IN_ENGLISH: readonly { readonly pattern: RegExp; readonly instead: string }[] = [
  {
    pattern: /\blog(ging)? in\b/iu,
    instead: '"sign in", que es el verbo que usa el resto del producto',
  },
  {
    pattern: /\blog(ging|ged)? out\b/iu,
    instead: '"sign out", que es el verbo que usa el resto del producto',
  },
];

describe('el vocabulario del ingles', () => {
  it('ningun texto usa una palabra que el producto no usa', () => {
    const offenders = [...en]
      .flatMap(([key, text]) =>
        BANNED_IN_ENGLISH.filter((rule) => rule.pattern.test(text)).map(
          (rule) => `${key}: usa ${rule.instead}`,
        ),
      )
      .sort();
    expect(offenders).toEqual([]);
  });
});

describe('las tablas de errores llevan solo codigos', () => {
  const TABLES = ['errors.engine.', 'errors.api.'];

  it('cada clave de errors.engine y errors.api es un code en snake_case', () => {
    const wrong = [...es.keys()]
      .filter((key) => TABLES.some((table) => key.startsWith(table)))
      .map((key) => ({ key, name: key.slice(key.lastIndexOf('.') + 1) }))
      .filter((entry) => !/^[a-z][a-z0-9]*(_[a-z0-9]+)*$/u.test(entry.name))
      .map((entry) => entry.key);
    expect(wrong).toEqual([]);
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
