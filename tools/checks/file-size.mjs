#!/usr/bin/env node
import { readFileSync } from 'node:fs';
import path from 'node:path';

import { report, run } from './lib/cli.mjs';
import { collectFiles, isExcluded, isSourceFile, isStyleFile, isTestFile } from './lib/scan.mjs';

export const LIMITS = { code: 300, test: 400 };

function countLines(absolutePath) {
  const content = readFileSync(absolutePath, 'utf8');
  if (content === '') return 0;
  const lines = content.split('\n');
  if (lines.at(-1) === '') lines.pop();
  return lines.length;
}

// El CSS entra con el limite del codigo: ADR 0005 lo pone bajo el mismo script.
export function checkFileSize({ root, staged }) {
  const files = collectFiles({ root, staged }).filter(
    (file) => (isSourceFile(file) || isStyleFile(file)) && !isExcluded(file),
  );
  const violations = [];
  for (const file of files) {
    const limit = isTestFile(file) ? LIMITS.test : LIMITS.code;
    const lines = countLines(path.join(root, file));
    if (lines > limit) {
      violations.push({
        file,
        message: `${String(lines)} lineas (limite ${String(limit)}). Partelo en piezas mas pequenas.`,
      });
    }
  }
  return { scanned: files.length, violations };
}

if (import.meta.main) {
  run((options) => report({ name: 'tamano de fichero', ...checkFileSize(options) }));
}
