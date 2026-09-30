#!/usr/bin/env node
import path from 'node:path';

import { report, run } from './lib/cli.mjs';
import { collectFiles, isExcluded, isSourceFile } from './lib/scan.mjs';
import { readLines } from './lib/text.mjs';

/**
 * Las palabras se arman por concatenacion para que este fichero no se delate
 * a si mismo. La comparacion distingue mayusculas: en castellano "todo" es una
 * palabra corriente y no puede disparar la comprobacion.
 *
 * Un ticket es una issue de GitHub (#123) o una tarjeta del tablero (RAM-123).
 * Es el mismo criterio que la regla ramal/ticketed-todo de ESLint; este script
 * existe porque ESLint no ve Rust, ni el CSS, ni los scripts y la configuracion.
 */
const WORDS = ['TO' + 'DO', 'FIX' + 'ME', 'XX' + 'X', 'HA' + 'CK'];
const MARKER = new RegExp(`\\b(?:${WORDS.join('|')})\\b`, 'u');
const TICKETED = new RegExp(`\\b(?:${WORDS.join('|')})\\((?:#\\d+|RAM-\\d+)\\)`, 'u');
const EXAMPLE = `${WORDS[0]}(#123) o ${WORDS[0]}(RAM-123)`;

/**
 * Ademas del codigo fuente, los ficheros con comentarios que ni ESLint ni este
 * script verian si no: el CSS de las apps, los scripts de shell y la
 * configuracion (Cargo, compose, workflows, lefthook). Sin ellos, un marcador en
 * un .css pasa en verde por todas las comprobaciones.
 */
const EXTRA_EXTENSIONS = new Set(['.css', '.sh', '.toml', '.yaml', '.yml']);

function isMarkerCandidate(file) {
  return isSourceFile(file) || EXTRA_EXTENSIONS.has(path.extname(file));
}

export function checkTodos({ root, staged }) {
  const files = collectFiles({ root, staged }).filter(
    (file) => isMarkerCandidate(file) && !isExcluded(file),
  );
  const violations = [];
  for (const file of files) {
    readLines(root, file).forEach((line, index) => {
      if (!MARKER.test(line) || TICKETED.test(line)) return;
      violations.push({
        file,
        line: index + 1,
        message: `marcador sin ticket: usa ${EXAMPLE} o borra la nota`,
      });
    });
  }
  return { scanned: files.length, violations };
}

if (import.meta.main) {
  run((options) => report({ name: 'marcadores sin ticket', ...checkTodos(options) }));
}
