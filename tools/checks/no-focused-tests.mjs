#!/usr/bin/env node
import { report, run } from './lib/cli.mjs';
import { collectFiles, isExcluded, isSourceFile, isTestFile } from './lib/scan.mjs';
import { hasNearbyComment, readLines } from './lib/text.mjs';

const FOCUSED_JS = /(?:\bdescribe|\bit|\btest|\bsuite|\bbench)(?:\.\w+)*\.only\s*\(/u;
const FOCUSED_SHORTHAND = /\bf(?:describe|it)\s*\(/u;
/**
 * Todas las formas de apagar un test en Vitest, Playwright y mocha (WebdriverIO):
 * skip, skipIf y runIf con su condicion, todo, fixme y los atajos xit/xdescribe.
 * Antes solo se miraban .skip y .todo, y un it.skipIf(true) pasaba en verde.
 */
const SKIPPED_JS =
  /(?:\bdescribe|\bit|\btest|\bsuite)(?:\.\w+)*\.(?:skip|skipIf|runIf|todo|fixme)\s*\(/u;
const SKIPPED_SHORTHAND = /\bx(?:describe|it|test)\s*\(/u;
/**
 * El motivo como argumento solo existe en Playwright: test.skip(condicion,
 * 'motivo') y test.fixme(condicion, 'motivo'). El primer argumento no puede
 * empezar por comilla (seria el titulo del test, no una condicion) y el motivo
 * no puede ir vacio. Un `, "` dentro del titulo ya no cuenta como motivo.
 */
const SKIPPED_JS_WITH_REASON =
  /\.(?:skip|fixme)\s*\(\s*(?![\x27"`])[^,()]*,\s*[\x27"`]\s*[^\s\x27"`]/u;
// Un #[ignore = ""] tampoco vale: el motivo tiene que decir algo.
const RUST_IGNORE = /#\[\s*ignore\s*(?:=\s*"\s*"\s*)?\]/u;

const JS_EXTENSIONS = /\.[cm]?[jt]sx?$/u;

function jsViolations(lines, file) {
  const found = [];
  lines.forEach((line, index) => {
    if (FOCUSED_JS.test(line) || FOCUSED_SHORTHAND.test(line)) {
      found.push({
        file,
        line: index + 1,
        message: 'test enfocado: quita el .only antes de commitear',
      });
      return;
    }
    const skipped = SKIPPED_JS.exec(line) ?? SKIPPED_SHORTHAND.exec(line);
    if (skipped === null) return;
    if (SKIPPED_JS_WITH_REASON.test(line)) return;
    if (hasNearbyComment(lines, index, skipped[0])) return;
    found.push({
      file,
      line: index + 1,
      message: 'test saltado sin motivo: pasa una razon o comentala',
    });
  });
  return found;
}

/** Una anotacion nombrada dentro de un comentario no apaga ningun test. */
function isInsideComment(line, at) {
  const before = line.slice(0, at);
  return before.includes('//') || /^\s*(\/\*|\*)/u.test(before);
}

// El marcador para buscar el comentario justificativo es el texto que de verdad
// casa, no un literal fijo.
function rustViolations(lines, file) {
  const found = [];
  lines.forEach((line, index) => {
    const match = RUST_IGNORE.exec(line);
    if (match === null) return;
    if (isInsideComment(line, match.index)) return;
    if (hasNearbyComment(lines, index, match[0])) return;
    found.push({
      file,
      line: index + 1,
      message: 'test ignorado sin motivo: usa #[ignore = "..."] o comenta por que',
    });
  });
  return found;
}

function violationsFor(root, file) {
  const lines = readLines(root, file);
  if (JS_EXTENSIONS.test(file)) return jsViolations(lines, file);
  if (file.endsWith('.rs')) return rustViolations(lines, file);
  return [];
}

export function checkFocusedTests({ root, staged }) {
  const files = collectFiles({ root, staged })
    .filter((file) => isSourceFile(file) && !isExcluded(file))
    .filter((file) => isTestFile(file) || file.endsWith('.rs'));
  const violations = files.flatMap((file) => violationsFor(root, file));
  return { scanned: files.length, violations };
}

if (import.meta.main) {
  run((options) => report({ name: 'tests enfocados', ...checkFocusedTests(options) }));
}
