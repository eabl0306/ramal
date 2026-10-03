#!/usr/bin/env node
import { existsSync, readFileSync } from 'node:fs';
import path from 'node:path';

import { report, run } from './lib/cli.mjs';
import { workspaceGlobs, workspacePackages } from './lib/workspace.mjs';

/**
 * Quien puede depender de quien entre los paquetes del monorepo. ESLint vigila
 * las capas dentro de un paquete; esto vigila el grafo entre paquetes, leyendo
 * los package.json (dependencies, devDependencies, peerDependencies y
 * optionalDependencies).
 *
 * `pure` marca los paquetes que no pueden tener ninguna dependencia externa en
 * runtime: el dominio, los textos y el protocolo tienen que poder usarse igual
 * en la web, en Tauri y en la extension sin arrastrar nada.
 *
 * repo-view si depende de bridge (architecture.md 5.2): habla con el motor por
 * el cliente tipado de @git-rudder/bridge, que solo conoce el puerto Transport. Asi la
 * vista del repositorio no sabe si habla con Tauri o con el host de VS Code: el
 * transporte concreto lo pone cada app.
 *
 * Anadir un paquete nuevo es anadir una linea aqui: un paquete que no esta en la
 * matriz hace fallar la comprobacion. Si es un paquete compartido de packages/,
 * tambien va en `ts_package_dirs` del justfile (ver packageListProblems).
 */
export const MATRIX = {
  '@git-rudder/domain': { dir: 'packages/domain', allowed: [], pure: true },
  '@git-rudder/i18n': { dir: 'packages/i18n', allowed: [], pure: true },
  '@git-rudder/bridge': { dir: 'packages/bridge', allowed: [], pure: true },
  '@git-rudder/ui': { dir: 'packages/ui', allowed: ['@git-rudder/i18n'] },
  '@git-rudder/repo-view': {
    dir: 'packages/repo-view',
    allowed: ['@git-rudder/ui', '@git-rudder/domain', '@git-rudder/i18n', '@git-rudder/bridge'],
  },
  '@git-rudder/api-client': { dir: 'packages/api-client', allowed: [] },
  '@git-rudder/web': {
    dir: 'apps/web',
    allowed: ['@git-rudder/domain', '@git-rudder/i18n', '@git-rudder/ui', '@git-rudder/api-client'],
  },
  '@git-rudder/desktop': {
    dir: 'apps/desktop',
    allowed: [
      '@git-rudder/domain',
      '@git-rudder/i18n',
      '@git-rudder/ui',
      '@git-rudder/repo-view',
      '@git-rudder/bridge',
    ],
  },
  'git-rudder': {
    dir: 'apps/vscode',
    allowed: [
      '@git-rudder/domain',
      '@git-rudder/i18n',
      '@git-rudder/ui',
      '@git-rudder/repo-view',
      '@git-rudder/bridge',
    ],
  },
  '@git-rudder/eslint-config': { dir: 'packages/eslint-config', allowed: [] },
  '@git-rudder/tsconfig': { dir: 'packages/tsconfig', allowed: [] },
};

/** Configuracion compartida: cualquiera puede usarla, pero solo para desarrollar. */
export const CONFIG_PACKAGES = new Set(['@git-rudder/eslint-config', '@git-rudder/tsconfig']);

const TOOLS_RULE = { allowed: [] };
const SECTIONS = ['dependencies', 'devDependencies', 'peerDependencies', 'optionalDependencies'];

function isInternal(name) {
  return name === 'git-rudder' || name.startsWith('@git-rudder/');
}

function ruleFor(pkg) {
  if (pkg.dir.startsWith('tools/')) return { rule: TOOLS_RULE, problems: [] };
  const rule = MATRIX[pkg.name];
  if (rule === undefined) {
    return {
      rule: null,
      problems: [`paquete desconocido "${pkg.name}": anadelo a MATRIX en package-graph.mjs`],
    };
  }
  const problems =
    rule.dir === pkg.dir ? [] : [`"${pkg.name}" tiene que vivir en ${rule.dir}, no en ${pkg.dir}`];
  return { rule, problems };
}

function internalProblem(pkg, rule, dep, section, version) {
  if (MATRIX[dep] === undefined) return `depende de un paquete desconocido "${dep}"`;
  if (!rule.allowed.includes(dep)) {
    const allowed = rule.allowed.length > 0 ? rule.allowed.join(', ') : 'ninguno';
    return `"${pkg.name}" no puede depender de "${dep}" (${section}). Permitidos: ${allowed}`;
  }
  if (!String(version).startsWith('workspace:')) {
    return `"${dep}" se enlaza con workspace:*, no con "${version}"`;
  }
  return null;
}

function dependencyProblem(pkg, rule, dep, section, version) {
  if (CONFIG_PACKAGES.has(dep)) {
    return section === 'devDependencies' ? null : `"${dep}" solo puede ir en devDependencies`;
  }
  if (isInternal(dep)) return internalProblem(pkg, rule, dep, section, version);
  if (rule.pure === true && section !== 'devDependencies') {
    return `"${pkg.name}" no admite dependencias externas en runtime: "${dep}" (${section})`;
  }
  return null;
}

function dependencyProblems(pkg, rule) {
  const problems = [];
  for (const section of SECTIONS) {
    for (const [dep, version] of Object.entries(pkg.manifest[section] ?? {})) {
      const problem = dependencyProblem(pkg, rule, dep, section, version);
      if (problem !== null) problems.push(problem);
    }
  }
  return problems;
}

function duplicateNames(packages) {
  const seen = new Map();
  const problems = [];
  for (const pkg of packages) {
    const first = seen.get(pkg.name);
    if (first === undefined) {
      seen.set(pkg.name, pkg.dir);
      continue;
    }
    problems.push({ file: `${pkg.dir}/package.json`, message: `nombre repetido con ${first}` });
  }
  return problems;
}

const PACKAGE_DIRS_VAR = /^ts_package_dirs\s*:=\s*"([^"]*)"/mu;

/** Las carpetas de `ts_package_dirs` en el justfile, o null si no hay lista. */
function listedPackageDirs(root) {
  const file = path.join(root, 'justfile');
  if (!existsSync(file)) return null;
  const match = PACKAGE_DIRS_VAR.exec(readFileSync(file, 'utf8'));
  return match === null ? null : (match[1] ?? '').split(/\s+/u).filter((dir) => dir !== '');
}

/**
 * `just packages-check` (lo unico que corre packages.yml) recorre solo las
 * carpetas de `ts_package_dirs`. Un paquete compartido que no este en esa lista
 * se queda sin lint, tipos ni cobertura en la CI, y la CI sigue en verde. Por
 * eso la lista tiene que coincidir con los paquetes de packages/ que no son
 * configuracion: ni uno de mas (renombrado o borrado) ni uno de menos.
 */
function packageListProblems(root, packages) {
  const shared = packages.filter(
    (pkg) => pkg.dir.startsWith('packages/') && !CONFIG_PACKAGES.has(pkg.name),
  );
  if (shared.length === 0) return [];
  const listed = listedPackageDirs(root);
  if (listed === null) {
    const message = 'falta ts_package_dirs: sin esa lista just packages-check no comprueba nada';
    return [{ file: 'justfile', message }];
  }
  const dirs = new Set(shared.map((pkg) => path.posix.basename(pkg.dir)));
  const problems = shared
    .filter((pkg) => !listed.includes(path.posix.basename(pkg.dir)))
    .map((pkg) => ({
      file: `${pkg.dir}/package.json`,
      message: 'no esta en ts_package_dirs del justfile: just packages-check no lo comprobaria',
    }));
  for (const dir of listed.filter((name) => !dirs.has(name))) {
    const message = `ts_package_dirs nombra packages/${dir}, que no es un paquete compartido`;
    problems.push({ file: 'justfile', message });
  }
  return problems;
}

export function checkPackageGraph({ root }) {
  const globs = workspaceGlobs(root);
  if (globs === null) {
    return { scanned: 0, violations: [{ file: 'pnpm-workspace.yaml', message: 'no existe' }] };
  }
  const packages = workspacePackages(root, globs).map((pkg) => ({
    ...pkg,
    name: String(pkg.manifest.name ?? pkg.dir),
  }));
  const violations = duplicateNames(packages);
  for (const pkg of packages) {
    const { rule, problems } = ruleFor(pkg);
    const all = rule === null ? problems : [...problems, ...dependencyProblems(pkg, rule)];
    for (const message of all) violations.push({ file: `${pkg.dir}/package.json`, message });
  }
  violations.push(...packageListProblems(root, packages));
  return { scanned: packages.length, violations };
}

if (import.meta.main) {
  run((options) => report({ name: 'grafo de paquetes', ...checkPackageGraph(options) }));
}
