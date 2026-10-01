#!/usr/bin/env node
import { readFileSync } from 'node:fs';
import path from 'node:path';

import { report, run } from './lib/cli.mjs';
import { blankRustNoise, lineOf, rustAttributes } from './lib/rust-source.mjs';
import { collectFiles, isExcluded, isSourceFile, isTestFile } from './lib/scan.mjs';

/**
 * Los limites de tamano y complejidad no se apagan en linea. Apagar un test exige
 * un motivo escrito, pero una directiva eslint-disable sin reglas o un
 * `#[allow(...)]` sobre los limites pasaban sin decir nada. Cambiar un limite es
 * una decision de equipo (ADR 0005) y se hace en clippy.toml o en
 * @ramal/eslint-config, no fichero a fichero.
 *
 * Es un script aparte y no una regla de ESLint o un lint de clippy a proposito:
 * una directiva eslint-disable sin reglas silencia tambien a la regla que la
 * vigila, y este script no depende de que nadie respete su propia configuracion.
 *
 * Busca las directivas por texto, no con un parser: una cadena que contenga el
 * texto de una directiva tambien cuenta. Los tests las arman por concatenacion.
 */

/** Lints de clippy que hacen cumplir los umbrales de clippy.toml. */
const RUST_LIMIT_LINTS = [
  'cognitive_complexity',
  // Nombre antiguo de cognitive_complexity: clippy lo sigue aceptando como alias.
  'cyclomatic_complexity',
  'excessive_nesting',
  'too_many_arguments',
  'too_many_lines',
];
/** Grupos que contienen alguno de esos lints: apagar el grupo los apaga. */
const RUST_LIMIT_GROUPS = ['all', 'complexity', 'nursery', 'pedantic', 'restriction'];
/**
 * En los tests de TypeScript max-lines-per-function esta apagado (tests.js). Lo
 * mismo vale para Rust: un test largo puede justificarse; la complejidad, no.
 */
const RUST_TEST_ALLOWED = new Set(['too_many_lines']);

/** Reglas de ESLint que hacen cumplir los limites y las capas. */
const ESLINT_LIMIT_RULES = new Set([
  'complexity',
  'max-depth',
  'max-lines',
  'max-lines-per-function',
  'max-nested-callbacks',
  'max-params',
  'sonarjs/cognitive-complexity',
]);
const ESLINT_LIMIT_PREFIXES = ['boundaries/'];

const RUST_LINT_REFERENCE = /\bclippy::(\w+)/gu;
const RUST_TOOL_ATTRIBUTE = /^\s*clippy::(?:cognitive|cyclomatic)_complexity\s*=/u;
const SUPPRESSION = /\b(?:allow|expect)\s*\(/u;

function rustViolations(text, file) {
  const testFile = isTestFile(file);
  const found = [];
  for (const { content, line } of rustAttributes(blankRustNoise(text))) {
    if (RUST_TOOL_ATTRIBUTE.test(content)) {
      found.push({ file, line, message: 'umbral de clippy cambiado en linea: va en clippy.toml' });
      continue;
    }
    if (!SUPPRESSION.test(content)) continue;
    for (const [, lint] of content.matchAll(RUST_LINT_REFERENCE)) {
      if (testFile && RUST_TEST_ALLOWED.has(lint)) continue;
      if (RUST_LIMIT_LINTS.includes(lint) || RUST_LIMIT_GROUPS.includes(lint)) {
        found.push({ file, line, message: `limite apagado con clippy::${lint}: parte el codigo` });
      }
    }
  }
  return found;
}

/**
 * Directivas de ESLint en comentarios de bloque y de linea. Las de bloque pueden
 * ocupar varias lineas. Un comentario de bloque que empieza por `eslint` a secas
 * es configuracion de reglas en linea (cambiar complexity a off, por ejemplo),
 * que cambia un limite igual que apagarlo.
 */
const ESLINT_DIRECTIVE =
  /(?:\/\*\s*(eslint-disable(?:-next-line|-line)?|eslint(?=\s))([\s\S]*?)\*\/|\/\/[ \t]*(eslint-disable(?:-next-line|-line)?)\b(.*))/gu;

function isLimitRule(rule) {
  return ESLINT_LIMIT_RULES.has(rule) || ESLINT_LIMIT_PREFIXES.some((p) => rule.startsWith(p));
}

function directiveProblems(kind, body) {
  if (kind === 'eslint') return ['configuracion de reglas en linea: va en eslint.config.js'];
  const [rulesPart = '', ...descriptionParts] = body.split('--');
  const rules = rulesPart
    .split(',')
    .map((rule) => rule.trim())
    .filter((rule) => rule !== '');
  if (rules.length === 0) return [`${kind} sin reglas apaga todas: nombra la regla y el motivo`];
  const problems = rules
    .filter((rule) => isLimitRule(rule))
    .map((rule) => `limite apagado con ${rule}: parte el codigo`);
  if (descriptionParts.join('--').trim() === '') {
    problems.push(`${kind} sin motivo: anade "-- <por que>" detras de las reglas`);
  }
  return problems;
}

function jsViolations(text, file) {
  const found = [];
  for (const match of text.matchAll(ESLINT_DIRECTIVE)) {
    const kind = match[1] ?? match[3] ?? '';
    const body = match[2] ?? match[4] ?? '';
    const line = lineOf(text, match.index);
    for (const message of directiveProblems(kind, body)) found.push({ file, line, message });
  }
  return found;
}

function violationsFor(root, file) {
  const text = readFileSync(path.join(root, file), 'utf8');
  return file.endsWith('.rs') ? rustViolations(text, file) : jsViolations(text, file);
}

export function checkLimitSuppressions({ root, staged }) {
  const files = collectFiles({ root, staged }).filter(
    (file) => isSourceFile(file) && !isExcluded(file),
  );
  const violations = files.flatMap((file) => violationsFor(root, file));
  return { scanned: files.length, violations };
}

if (import.meta.main) {
  run((options) => report({ name: 'limites apagados', ...checkLimitSuppressions(options) }));
}
