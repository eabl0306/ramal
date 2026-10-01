import type { Locale } from './locale';
import enAi from './locales/en/ai.json';
import enCommon from './locales/en/common.json';
import enConflict from './locales/en/conflict.json';
import enDialog from './locales/en/dialog.json';
import enDiff from './locales/en/diff.json';
import enErrors from './locales/en/errors.json';
import enGraph from './locales/en/graph.json';
import enMenu from './locales/en/menu.json';
import enRebase from './locales/en/rebase.json';
import enRepos from './locales/en/repos.json';
import enSettings from './locales/en/settings.json';
import enStaging from './locales/en/staging.json';
import enTeam from './locales/en/team.json';
import enToast from './locales/en/toast.json';
import enVscode from './locales/en/vscode.json';
import enWeb from './locales/en/web.json';
import esAi from './locales/es/ai.json';
import esCommon from './locales/es/common.json';
import esConflict from './locales/es/conflict.json';
import esDialog from './locales/es/dialog.json';
import esDiff from './locales/es/diff.json';
import esErrors from './locales/es/errors.json';
import esGraph from './locales/es/graph.json';
import esMenu from './locales/es/menu.json';
import esRebase from './locales/es/rebase.json';
import esRepos from './locales/es/repos.json';
import esSettings from './locales/es/settings.json';
import esStaging from './locales/es/staging.json';
import esTeam from './locales/es/team.json';
import esToast from './locales/es/toast.json';
import esVscode from './locales/es/vscode.json';
import esWeb from './locales/es/web.json';

/**
 * Los diccionarios, un JSON por idioma y espacio de nombres. El castellano es la
 * fuente de verdad: de su forma salen las claves tipadas.
 */
const es = {
  common: esCommon,
  repos: esRepos,
  graph: esGraph,
  staging: esStaging,
  diff: esDiff,
  conflict: esConflict,
  rebase: esRebase,
  menu: esMenu,
  dialog: esDialog,
  toast: esToast,
  settings: esSettings,
  ai: esAi,
  team: esTeam,
  errors: esErrors,
  vscode: esVscode,
  web: esWeb,
};

const en = {
  common: enCommon,
  repos: enRepos,
  graph: enGraph,
  staging: enStaging,
  diff: enDiff,
  conflict: enConflict,
  rebase: enRebase,
  menu: enMenu,
  dialog: enDialog,
  toast: enToast,
  settings: enSettings,
  ai: enAi,
  team: enTeam,
  errors: enErrors,
  vscode: enVscode,
  web: enWeb,
};

export type Catalog = typeof es;

/**
 * Verdadero solo si los dos tipos tienen exactamente las mismas claves a todos
 * los niveles: cada uno tiene que poder asignarse al otro.
 */
type SameShape<A, B> = [A] extends [B] ? ([B] extends [A] ? true : false) : false;

/**
 * Si al ingles le falta una clave del castellano, o le sobra una, el tipo del
 * ingles se vuelve `never` y esta asignacion no compila. Es la paridad en tiempo
 * de compilacion; test/parity.test.ts la repite en ejecucion junto con la de
 * parametros y plurales, que el tipo de un JSON no puede ver.
 */
export const CATALOGS: {
  readonly es: Catalog;
  readonly en: SameShape<typeof en, Catalog> extends true ? Catalog : never;
} = { es, en };

export const NAMESPACES = Object.keys(es) as (keyof Catalog)[];

export function catalogFor(locale: Locale): Catalog {
  return CATALOGS[locale];
}
