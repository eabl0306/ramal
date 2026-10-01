import { existsSync, readdirSync, readFileSync } from 'node:fs';
import path from 'node:path';

import { IGNORED_DIRS } from './scan.mjs';
import { parseTomlLite, tomlStrings, tomlValue } from './toml.mjs';

export function readToml(root, relativePath) {
  return parseTomlLite(readFileSync(path.join(root, relativePath), 'utf8'));
}

function expandMember(root, member) {
  if (!member.endsWith('/*')) return [member];
  const parent = member.slice(0, -2);
  const absolute = path.join(root, parent);
  if (!existsSync(absolute)) return [];
  return readdirSync(absolute, { withFileTypes: true })
    .filter((entry) => entry.isDirectory() && !IGNORED_DIRS.has(entry.name))
    .map((entry) => `${parent}/${entry.name}`)
    .filter((dir) => existsSync(path.join(root, dir, 'Cargo.toml')));
}

/**
 * Directorios de los miembros del workspace de Cargo de la raiz, con los globs
 * de un nivel ("crates/*") ya expandidos. Vacio si no hay workspace.
 */
export function cargoMembers(root) {
  if (!existsSync(path.join(root, 'Cargo.toml'))) return [];
  const members = tomlStrings(tomlValue(readToml(root, 'Cargo.toml'), 'workspace', 'members'));
  return members.flatMap((member) => expandMember(root, member));
}
