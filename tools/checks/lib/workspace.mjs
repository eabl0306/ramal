import { existsSync, readdirSync, readFileSync } from 'node:fs';
import path from 'node:path';

import { IGNORED_DIRS, toPosix } from './scan.mjs';

const LIST_ITEM = /^\s+-\s+(.+?)\s*$/u;

function unquote(value) {
  return value.replace(/^(['"])(.*)\1$/u, '$2');
}

/**
 * Lee los globs de `packages:` en pnpm-workspace.yaml. No es un parser de YAML:
 * el bloque es una lista plana y los scripts de tools/checks no tienen
 * dependencias. Devuelve null si el fichero no existe.
 */
export function workspaceGlobs(root) {
  const file = path.join(root, 'pnpm-workspace.yaml');
  if (!existsSync(file)) return null;
  const globs = [];
  let inPackages = false;
  for (const line of readFileSync(file, 'utf8').split('\n')) {
    if (/^packages:\s*$/u.test(line)) {
      inPackages = true;
      continue;
    }
    if (!inPackages || /^\s*(#.*)?$/u.test(line)) continue;
    const item = LIST_ITEM.exec(line);
    if (item === null) break;
    globs.push(unquote(item[1] ?? ''));
  }
  return globs;
}

function subdirectories(absoluteDir) {
  if (!existsSync(absoluteDir)) return [];
  return readdirSync(absoluteDir, { withFileTypes: true })
    .filter((entry) => entry.isDirectory() && !IGNORED_DIRS.has(entry.name))
    .map((entry) => entry.name);
}

function descendants(root, relativeDir) {
  const found = [relativeDir];
  for (const name of subdirectories(path.join(root, relativeDir))) {
    found.push(...descendants(root, path.posix.join(relativeDir, name)));
  }
  return found;
}

/** Expande un glob de pnpm (`a/b`, `a/*`, `a/**`) a directorios existentes. */
function expand(root, relativeDir, segments) {
  if (segments.length === 0) return [relativeDir];
  const [segment, ...rest] = segments;
  if (segment === '**') {
    return descendants(root, relativeDir).flatMap((dir) => expand(root, dir, rest));
  }
  if (segment === '*') {
    return subdirectories(path.join(root, relativeDir)).flatMap((name) =>
      expand(root, path.posix.join(relativeDir, name), rest),
    );
  }
  const next = path.posix.join(relativeDir, segment ?? '');
  return existsSync(path.join(root, next)) ? expand(root, next, rest) : [];
}

function directoriesFor(root, globs) {
  const include = globs.filter((glob) => !glob.startsWith('!'));
  const exclude = new Set(
    globs
      .filter((glob) => glob.startsWith('!'))
      .flatMap((glob) => expand(root, '.', glob.slice(1).split('/'))),
  );
  const dirs = include.flatMap((glob) => expand(root, '.', glob.split('/')));
  return [...new Set(dirs)].filter((dir) => !exclude.has(dir));
}

/**
 * Paquetes del workspace: los directorios que casan con los globs y tienen un
 * package.json. Un glob que todavia no casa con nada (una app que aun no existe)
 * no es un error.
 */
export function workspacePackages(root, globs) {
  return directoriesFor(root, globs)
    .filter((dir) => existsSync(path.join(root, dir, 'package.json')))
    .map((dir) => ({
      dir: toPosix(path.normalize(dir)),
      manifest: JSON.parse(readFileSync(path.join(root, dir, 'package.json'), 'utf8')),
    }))
    .sort((a, b) => a.dir.localeCompare(b.dir));
}
