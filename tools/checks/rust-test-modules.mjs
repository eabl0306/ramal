#!/usr/bin/env node
import { readFileSync } from 'node:fs';
import path from 'node:path';

import { report, run } from './lib/cli.mjs';
import { blankRustNoise, lineOf } from './lib/rust-source.mjs';
import { collectFiles, isExcluded } from './lib/scan.mjs';

/**
 * Los tests de Rust van en su propio fichero (`#[cfg(test)] mod tests;` mas
 * `<modulo>/tests.rs`), nunca en linea dentro del codigo de produccion.
 *
 * Existe por la cobertura: cargo-llvm-cov deja fuera los tests.rs, pero un
 * `mod tests { ... }` en linea cuenta sus lineas como codigo cubierto. En el
 * nucleo de la API, mas de la mitad de las lineas medidas eran cuerpos de test,
 * y un caso de uso podia quedarse al 0% con el umbral del 80% en verde.
 *
 * Las ayudas de `src/testing/` quedan fuera: la cobertura las ignora entera.
 */
const INLINE_TEST_MODULE =
  /#\[\s*cfg\s*\(\s*(?:all\s*\(\s*)?test\b[^\]]*\]\s*(?:#!?\[[^\]]*\]\s*)*(?:pub(?:\s*\([^)]*\))?\s+)?mod\s+(\w+)\s*\{/gu;

function isProductionRust(file) {
  if (!file.endsWith('.rs') || isExcluded(file)) return false;
  if (!/(^|\/)src\//u.test(file) || /(^|\/)src\/testing\//u.test(file)) return false;
  return path.basename(file) !== 'tests.rs';
}

function violationsFor(root, file) {
  const clean = blankRustNoise(readFileSync(path.join(root, file), 'utf8'));
  return [...clean.matchAll(INLINE_TEST_MODULE)].map((match) => ({
    file,
    line: lineOf(clean, match.index),
    message: `mod ${match[1] ?? 'tests'} en linea: muevelo a su tests.rs para que la cobertura no lo cuente`,
  }));
}

export function checkRustTestModules({ root, staged }) {
  const files = collectFiles({ root, staged }).filter(isProductionRust);
  return { scanned: files.length, violations: files.flatMap((file) => violationsFor(root, file)) };
}

if (import.meta.main) {
  run((options) => report({ name: 'modulos de test de Rust', ...checkRustTestModules(options) }));
}
