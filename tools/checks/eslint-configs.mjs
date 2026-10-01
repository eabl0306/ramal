#!/usr/bin/env node
import { existsSync } from 'node:fs';
import path from 'node:path';

import { report, run } from './lib/cli.mjs';
import { collectFiles, isExcluded } from './lib/scan.mjs';
import { workspaceGlobs, workspacePackages } from './lib/workspace.mjs';

/**
 * Todo el TypeScript del repo lo linta la configuracion de su paquete. El
 * eslint.config.js de la raiz ignora apps/ y packages/ a proposito (cada paquete
 * trae el suyo, con tipos, React y capas), asi que un paquete con TypeScript y
 * sin eslint.config.js propio no lo linta nadie: ESLint sube hasta la raiz, ve
 * sus ficheros ignorados y sale en verde. Lo mismo un .ts que no cae dentro de
 * ningun paquete del workspace.
 */
const TYPESCRIPT = /\.(?:[cm]?ts|tsx)$/u;
const DECLARATION = /\.d\.[cm]?ts$/u;
/** Los nombres que ESLint 10 busca como configuracion plana. */
const CONFIG_NAMES = ['js', 'mjs', 'cjs', 'ts', 'mts', 'cts'].map((ext) => `eslint.config.${ext}`);

function isLintableTypescript(file) {
  return TYPESCRIPT.test(file) && !DECLARATION.test(file) && !isExcluded(file);
}

function owningPackage(file, packageDirs) {
  return packageDirs
    .filter((dir) => file.startsWith(`${dir}/`))
    .sort((a, b) => b.length - a.length)[0];
}

function hasOwnConfig(root, dir) {
  return CONFIG_NAMES.some((name) => existsSync(path.join(root, dir, name)));
}

export function checkEslintConfigs({ root }) {
  const globs = workspaceGlobs(root) ?? [];
  const packageDirs = workspacePackages(root, globs).map((pkg) => pkg.dir);
  const files = collectFiles({ root, staged: false }).filter(isLintableTypescript);
  const withTypescript = new Map();
  const violations = [];
  for (const file of files) {
    const dir = owningPackage(file, packageDirs);
    if (dir === undefined) {
      violations.push({
        file,
        message: 'TypeScript fuera de cualquier paquete del workspace: no lo linta nadie',
      });
    } else if (!withTypescript.has(dir)) {
      withTypescript.set(dir, file);
    }
  }
  for (const [dir, example] of withTypescript) {
    if (hasOwnConfig(root, dir)) continue;
    violations.push({
      file: dir,
      message: `tiene TypeScript (${example}) y no tiene eslint.config.js propio: sus ficheros no se lintan`,
    });
  }
  return { scanned: withTypescript.size, violations };
}

if (import.meta.main) {
  run((options) => report({ name: 'configuraciones de eslint', ...checkEslintConfigs(options) }));
}
