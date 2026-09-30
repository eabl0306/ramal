#!/usr/bin/env node
import { report, run } from './lib/cli.mjs';
import { collectFiles, isExcluded, isSourceFile, isTestFile } from './lib/scan.mjs';
import { hasNearbyComment, readLines } from './lib/text.mjs';

const FOCUSED_JS = /(?:\bdescribe|\bit|\btest|\bsuite|\bbench)(?:\.\w+)*\.only\s*\(/u;
const FOCUSED_SHORTHAND = /\bf(?:describe|it)\s*\(/u;
const SKIPPED_JS = /(?:\bdescribe|\bit|\btest|\bsuite)(?:\.\w+)*\.(?:skip|todo)\s*\(/u;
const SKIPPED_JS_WITH_REASON = /\.(?:skip|todo)\s*\([^)]*,\s*['"`]/u;
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
    if (!SKIPPED_JS.test(line)) return;
    if (SKIPPED_JS_WITH_REASON.test(line)) return;
    if (hasNearbyComment(lines, index, '.skip')) return;
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
