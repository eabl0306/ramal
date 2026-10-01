#!/usr/bin/env node
import { existsSync, readFileSync } from 'node:fs';
import path from 'node:path';

import { cargoMembers } from './lib/cargo.mjs';
import { report, run } from './lib/cli.mjs';
import { collectFiles } from './lib/scan.mjs';

/**
 * Existe por un fallo real: se anadio tools/rust-complexity al workspace de
 * Cargo, `just check` paso en verde y la imagen de la API dejo de construir en
 * la CI. Su contexto es una lista blanca (apps/api/Dockerfile.dockerignore) y
 * cargo necesita el manifiesto de TODOS los miembros para resolver el
 * workspace, aunque solo compile ramal-api.
 *
 * Regla: todo `<Dockerfile>.dockerignore` cuyo Dockerfile ejecuta cargo y que
 * empieza excluyendolo todo ("*") tiene que readmitir cada miembro del
 * workspace, por su ruta o por un directorio que lo contenga.
 */

function readLines(root, file) {
  return readFileSync(path.join(root, file), 'utf8')
    .split('\n')
    .map((line) => line.trim())
    .filter((line) => line !== '' && !line.startsWith('#'));
}

function buildsCargo(root, ignoreFile) {
  const dockerfile = ignoreFile.slice(0, -'.dockerignore'.length);
  const absolute = path.join(root, dockerfile);
  return existsSync(absolute) && /\bcargo\s/u.test(readFileSync(absolute, 'utf8'));
}

function allowedPrefixes(lines) {
  return lines
    .filter((line) => line.startsWith('!'))
    .map((line) =>
      line
        .slice(1)
        .replace(/^\/+/u, '')
        .replace(/\/(\*\*)?$/u, ''),
    );
}

function isAllowed(dir, prefixes) {
  return prefixes.some((prefix) => dir === prefix || dir.startsWith(`${prefix}/`));
}

export function checkDockerContext({ root }) {
  const members = cargoMembers(root);
  const ignoreFiles = collectFiles({ root, staged: false }).filter(
    (file) => file.endsWith('Dockerfile.dockerignore') && buildsCargo(root, file),
  );
  const violations = [];
  for (const file of ignoreFiles) {
    const lines = readLines(root, file);
    if (lines[0] !== '*') continue;
    const prefixes = allowedPrefixes(lines);
    for (const dir of members.filter((member) => !isAllowed(member, prefixes))) {
      violations.push({
        file,
        message: `falta "!${dir}": sin el manifiesto de ese miembro cargo no resuelve el workspace dentro de la imagen`,
      });
    }
  }
  return { scanned: ignoreFiles.length, violations };
}

if (import.meta.main) {
  run((options) => report({ name: 'contexto de docker', ...checkDockerContext(options) }));
}
