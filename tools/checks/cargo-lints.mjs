#!/usr/bin/env node
import { existsSync, readFileSync } from 'node:fs';
import path from 'node:path';

import { cargoMembers, readToml } from './lib/cargo.mjs';
import { report, run } from './lib/cli.mjs';
import { collectFiles } from './lib/scan.mjs';
import { tomlValue } from './lib/toml.mjs';

/**
 * Todos los crates del workspace pasan por los mismos lints y los mismos
 * umbrales. Lo que vigila, y por que:
 *
 * - Cada miembro declara `[lints] workspace = true`. Sin esa linea el crate no
 *   hereda [workspace.lints] y clippy se salta en silencio todos los limites
 *   (complejidad, tamano de funcion, unwrap, indexing...) aunque la CI siga en
 *   verde.
 * - Ningun clippy.toml anidado cambia los umbrales de la raiz. Clippy usa el
 *   clippy.toml mas cercano y no fusiona: uno que no repite un umbral vuelve al
 *   valor por defecto de clippy (100 lineas, cognitiva 25), mas laxo.
 * - La configuracion de cargo de la raiz no redirige clippy.toml
 *   (CLIPPY_CONF_DIR) ni apaga lints con rustflags.
 */

function inheritsWorkspaceLints(entries) {
  if (tomlValue(entries, 'lints', 'workspace') === 'true') return true;
  if (tomlValue(entries, '', 'lints.workspace') === 'true') return true;
  return /\bworkspace\s*=\s*true\b/u.test(tomlValue(entries, '', 'lints') ?? '');
}

function memberViolations(root) {
  const dirs = cargoMembers(root);
  const violations = [];
  for (const dir of dirs) {
    const manifest = `${dir}/Cargo.toml`;
    if (!existsSync(path.join(root, manifest))) {
      violations.push({ file: manifest, message: 'miembro del workspace sin Cargo.toml' });
      continue;
    }
    if (inheritsWorkspaceLints(readToml(root, manifest))) continue;
    violations.push({
      file: manifest,
      message: 'falta "[lints] workspace = true": sin ella clippy no aplica ningun limite',
    });
  }
  return { checked: dirs.length, violations };
}

function thresholds(entries) {
  return entries.filter((entry) => entry.table === '' && entry.key.endsWith('-threshold'));
}

function clippyViolations(root, files) {
  const nested = files.filter(
    (file) => file.includes('/') && /^\.?clippy\.toml$/u.test(path.basename(file)),
  );
  const expected = thresholds(readToml(root, 'clippy.toml'));
  const violations = [];
  for (const file of nested) {
    const entries = readToml(root, file);
    for (const { key, value } of expected) {
      const actual = tomlValue(entries, '', key);
      if (actual === value) continue;
      const found = actual === undefined ? 'no lo repite' : `pone ${actual}`;
      violations.push({
        file,
        message: `${key} tiene que ser ${value} como en la raiz y ${found}: clippy no fusiona ficheros`,
      });
    }
  }
  return { checked: nested.length, violations };
}

const CARGO_CONFIGS = ['.cargo/config.toml', '.cargo/config'];
const LOOSENING = /CLIPPY_CONF_DIR|cap-lints|["'\s](?:-A|--allow)\b/u;

function cargoConfigViolations(root) {
  const present = CARGO_CONFIGS.filter((file) => existsSync(path.join(root, file)));
  const violations = present
    .filter((file) => LOOSENING.test(readFileSync(path.join(root, file), 'utf8')))
    .map((file) => ({
      file,
      message: 'no redirijas clippy.toml ni apagues lints desde la configuracion de cargo',
    }));
  return { checked: present.length, violations };
}

export function checkCargoLints({ root }) {
  if (!existsSync(path.join(root, 'Cargo.toml'))) return { scanned: 0, violations: [] };
  const files = collectFiles({ root, staged: false });
  const parts = [
    memberViolations(root),
    clippyViolations(root, files),
    cargoConfigViolations(root),
  ];
  return {
    scanned: parts.reduce((total, part) => total + part.checked, 0),
    violations: parts.flatMap((part) => part.violations),
  };
}

if (import.meta.main) {
  run((options) => report({ name: 'lints de cargo', ...checkCargoLints(options) }));
}
