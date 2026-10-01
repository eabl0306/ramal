import type { ConflictFile, ConflictLabels } from '../conflict/model';

/**
 * Los dos ficheros en conflicto del prototipo (ramal-data.js, CF), con los que
 * el analisis de datos verifico las salidas esperadas.
 */

export const LABELS: ConflictLabels = { ours: 'feature/graph-ui', theirs: 'hotfix/login' };

export const SESSION_FILE: ConflictFile = {
  path: 'src/auth/session.ts',
  segments: [
    {
      kind: 'context',
      lines: [
        'import { api } from "../net/api";',
        'import { store } from "./store";',
        'export async function refreshSession(acc: Account) {',
      ],
    },
    {
      kind: 'hunk',
      ours: ['  const token = await store.get(acc.id);', '  if (!token) return null;'],
      theirs: [
        '  const token = await vault.read(acc.id, { decrypt: true });',
        '  if (!token || token.expired) {',
        '    return reauth(acc);',
        '  }',
      ],
    },
    { kind: 'context', lines: ['', '  const res = await api(acc).post("/oauth/refresh", {'] },
    {
      kind: 'hunk',
      ours: ['    refresh_token: token.refresh,', '    timeout: 5000,'],
      theirs: ['    refresh_token: token.refresh,', '    scope: acc.scopes.join(" "),'],
    },
    { kind: 'context', lines: ['    client_id: acc.clientId,', '  });', '}'] },
  ],
};

export const I18N_FILE: ConflictFile = {
  path: 'src/i18n/es.json',
  segments: [
    {
      kind: 'context',
      lines: ['{', '  "graph.zoom": "Zoom",', '  "graph.reset": "Restablecer vista",'],
    },
    {
      kind: 'hunk',
      ours: ['  "auth.expired": "Sesión caducada",'],
      theirs: [
        '  "auth.expired": "Tu sesión ha expirado. Vuelve a iniciar sesión.",',
        '  "auth.reauth": "Iniciar sesión de nuevo",',
      ],
    },
    { kind: 'context', lines: ['  "auth.retry": "Reintentar"', '}'] },
  ],
};

/** Un borrado frente a una modificacion: el lado de B esta vacio. */
export const DELETED_FILE: ConflictFile = {
  path: 'src/legacy.ts',
  segments: [
    { kind: 'context', lines: ['// cabecera'] },
    { kind: 'hunk', ours: ['export const legacy = 2;'], theirs: [] },
  ],
};
