#!/usr/bin/env node
import { existsSync, readFileSync } from 'node:fs';
import path from 'node:path';

import { report, run } from './lib/cli.mjs';
import { collectFiles } from './lib/scan.mjs';

/**
 * La maquina, la CI y las imagenes de Docker compilan con las mismas versiones
 * (ADR 0005). La CI las lee de rust-toolchain.toml, .nvmrc y el
 * `packageManager` de package.json; los Dockerfile las repiten en un ARG porque
 * Docker no puede leer esos ficheros. Dependabot sube el fichero y no el ARG, y
 * la imagen de produccion seguia compilando con el rustc viejo con la CI en
 * verde. Esto falla mientras no vayan a la par.
 *
 * Para no dar un verde falso: un Dockerfile que usa la herramienta sin el ARG,
 * o con la version escrita a mano en el FROM o en `pnpm@`, tambien falla, y no
 * encontrar ningun Dockerfile es un error.
 */

const DOCKERFILE = /(^|\/)(Dockerfile(\.[^./]+)?|[^/]+\.Dockerfile)$/u;

function majorOf(version) {
  return /^v?(\d+)/u.exec(version)?.[1] ?? null;
}

const TOOLS = [
  {
    name: 'Rust',
    arg: 'RUST_VERSION',
    source: 'rust-toolchain.toml',
    uses: [/^FROM\s+rust:/imu, /rustup-init/u],
    pinned: /^FROM\s+rust:(\S+)/gimu,
    matches: (value, versions) => value === versions.rust,
  },
  {
    name: 'Node',
    arg: 'NODE_VERSION',
    source: '.nvmrc (la mayor)',
    uses: [/^FROM\s+node:/imu, /nodejs\.org\/dist/u],
    pinned: /^FROM\s+node:(\S+)/gimu,
    matches: (value, versions) => majorOf(value) === majorOf(versions.node),
  },
  {
    name: 'pnpm',
    arg: 'PNPM_VERSION',
    source: 'packageManager de package.json',
    uses: [/pnpm@/u],
    pinned: /pnpm@([^\s"']+)/gu,
    matches: (value, versions) => value === versions.pnpm,
  },
];

function readText(root, file) {
  const absolute = path.join(root, file);
  return existsSync(absolute) ? readFileSync(absolute, 'utf8') : null;
}

function packageManagerVersion(root) {
  const text = readText(root, 'package.json');
  if (text === null) return null;
  const field = String(JSON.parse(text).packageManager ?? '');
  return field.startsWith('pnpm@') ? (field.slice('pnpm@'.length).split('+')[0] ?? null) : null;
}

/** Las versiones de referencia, y un problema por cada una que no se puede leer. */
function readVersions(root) {
  const rust = /^channel\s*=\s*"([^"]+)"/mu.exec(readText(root, 'rust-toolchain.toml') ?? '')?.[1];
  const node = readText(root, '.nvmrc')?.trim() ?? '';
  const versions = { rust: rust ?? null, node, pnpm: packageManagerVersion(root) };
  const problems = [];
  if (versions.rust === null) {
    problems.push({ file: 'rust-toolchain.toml', message: 'no fija `channel = "<version>"`' });
  }
  if (majorOf(node) === null) {
    problems.push({ file: '.nvmrc', message: 'no fija una version numerica de Node' });
  }
  if (versions.pnpm === null) {
    problems.push({ file: 'package.json', message: 'no fija packageManager "pnpm@<version>"' });
  }
  return { versions, problems };
}

/** Las lineas sin comentarios, numeradas desde 1, para que los avisos apunten bien. */
function instructions(text) {
  return text
    .split('\n')
    .map((content, index) => ({ content, line: index + 1 }))
    .filter(({ content }) => !content.trimStart().startsWith('#'));
}

function argProblems(file, lines, tool, versions) {
  const declaration = new RegExp(`^\\s*ARG\\s+${tool.arg}=["']?([^"'\\s]+)`, 'u');
  const problems = [];
  let declared = 0;
  for (const { content, line } of lines) {
    const value = declaration.exec(content)?.[1];
    if (value === undefined) continue;
    declared += 1;
    if (tool.matches(value, versions)) continue;
    const message = `ARG ${tool.arg}=${value} no va a la par con ${tool.source}`;
    problems.push({ file, line, message: `${message} (${describe(tool, versions)})` });
  }
  const used = tool.uses.some((pattern) => lines.some(({ content }) => pattern.test(content)));
  if (used && declared === 0) {
    const message = `usa ${tool.name} sin ARG ${tool.arg}=<version>: no se puede comprobar`;
    problems.push({ file, message });
  }
  return problems;
}

function describe(tool, versions) {
  if (tool.arg === 'RUST_VERSION') return versions.rust;
  if (tool.arg === 'NODE_VERSION') return `mayor ${String(majorOf(versions.node))}`;
  return versions.pnpm;
}

/** Un FROM rust:/node: o un pnpm@ con la version escrita a mano se salta el ARG. */
function pinnedProblems(file, lines, tool) {
  const problems = [];
  for (const { content, line } of lines) {
    for (const match of content.matchAll(tool.pinned)) {
      const reference = match[1] ?? '';
      if (reference.includes(`\${${tool.arg}}`) || reference.includes(`$${tool.arg}`)) continue;
      problems.push({ file, line, message: `${match[0]} no usa \${${tool.arg}}` });
    }
  }
  return problems;
}

function dockerfileProblems(root, file, versions) {
  const lines = instructions(readFileSync(path.join(root, file), 'utf8'));
  return TOOLS.flatMap((tool) => [
    ...argProblems(file, lines, tool, versions),
    ...pinnedProblems(file, lines, tool),
  ]);
}

export function checkToolchainVersions({ root }) {
  const files = collectFiles({ root, staged: false }).filter(
    (file) => DOCKERFILE.test(file) && !file.endsWith('.dockerignore'),
  );
  const { versions, problems } = readVersions(root);
  if (files.length === 0) {
    problems.push({ file: '.', message: 'no hay ningun Dockerfile: la comprobacion no mira nada' });
  }
  if (problems.length > 0) return { scanned: files.length, violations: problems };
  const violations = files.flatMap((file) => dockerfileProblems(root, file, versions));
  return { scanned: files.length, violations };
}

if (import.meta.main) {
  run((options) =>
    report({ name: 'versiones de las herramientas', ...checkToolchainVersions(options) }),
  );
}
