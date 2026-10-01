#!/usr/bin/env node
// Genera src/generated/params.ts: los parametros de cada clave del castellano.
//
// Existe porque TypeScript importa los textos de un JSON como `string`, asi que
// no puede ver los marcadores {nombre} que llevan dentro. El test de paridad
// recalcula esta tabla y falla si el fichero no esta al dia.
//
//   corepack pnpm --filter @ramal/i18n generate
import { readdirSync, readFileSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const sourceDir = path.join(root, 'src', 'locales', 'es');
const target = path.join(root, 'src', 'generated', 'params.ts');

const PLACEHOLDER = /\{([A-Za-z][A-Za-z0-9]*)\}/gu;
const PLURAL = /_(zero|one|two|few|many|other)$/u;

function flatten(tree, prefix, out) {
  for (const [key, value] of Object.entries(tree)) {
    const full = prefix === '' ? key : `${prefix}.${key}`;
    if (typeof value === 'string') out.set(full, value);
    else flatten(value, full, out);
  }
  return out;
}

function readCatalog() {
  const messages = new Map();
  for (const file of readdirSync(sourceDir)
    .filter((name) => name.endsWith('.json'))
    .sort()) {
    const tree = JSON.parse(readFileSync(path.join(sourceDir, file), 'utf8'));
    flatten(tree, path.basename(file, '.json'), messages);
  }
  return messages;
}

/** Una clave plural junta los parametros de todas sus formas, y siempre lleva count. */
function paramsByKey(messages) {
  const table = new Map();
  for (const [key, text] of messages) {
    const base = key.replace(PLURAL, '');
    const names = table.get(base) ?? new Set();
    if (base !== key) names.add('count');
    for (const match of text.matchAll(PLACEHOLDER)) names.add(match[1]);
    table.set(base, names);
  }
  return [...table]
    .filter(([, names]) => names.size > 0)
    .sort(([a], [b]) => (a < b ? -1 : 1))
    .map(([key, names]) => [key, [...names].sort()]);
}

function render(entries) {
  const lines = entries.map(
    ([key, names]) => `  '${key}': [${names.map((name) => `'${name}'`).join(', ')}],`,
  );
  return [
    '// Generado por scripts/generate-params.mjs desde src/locales/es. No editar a mano:',
    '// corepack pnpm --filter @ramal/i18n generate',
    'export const PARAMS = {',
    ...lines,
    '} as const;',
    '',
  ].join('\n');
}

writeFileSync(target, render(paramsByKey(readCatalog())));
process.stdout.write(`generate-params: ${path.relative(root, target)} al dia\n`);
