#!/usr/bin/env node
import { existsSync, readFileSync, readdirSync, statSync } from 'node:fs';
import path from 'node:path';

import { report, run } from './lib/cli.mjs';
import { IGNORED_DIRS, toPosix } from './lib/scan.mjs';

/**
 * Enlaces y anclas entre documentos. Las revisiones encontraron documentos que
 * se contradecian y anclas rotas, y el renombrado a Git Rudder movio cada ancla
 * que llevaba el nombre del producto: un enlace roto no se ve leyendo el diff.
 *
 * Mira tres cosas en el Markdown de docs/ y en el de la raiz:
 *
 * - un enlace relativo a un fichero que no existe,
 * - una ancla que ningun titulo del documento de destino genera,
 * - un ADR citado como "ADR 0019" que no esta en docs/adr/.
 *
 * No toca las URL absolutas (nadie quiere una comprobacion que pida red) ni
 * docs/design/handoff/, que es la entrega del diseno y no codigo del producto.
 * Tampoco mira los enlaces dentro de un bloque de codigo: ahi un corchete es un
 * ejemplo, no un enlace.
 */

/** Markdown del producto. docs/design/handoff es material de referencia. */
const EXCLUDED = [/^docs\/design\/handoff\//u];

/** Un enlace o imagen de Markdown: el destino entre parentesis. */
const LINK = /!?\[(?:[^\]\\]|\\.)*\]\(([^()\s]+)\)/gu;

/** Una cita de ADR en prosa: "ADR 0019", "ADRs 0019" o "el ADR 0019". */
const ADR_MENTION = /\bADRs?\s+(\d{4})\b/gu;

function markdownFiles(root) {
  const found = [];
  const docs = path.join(root, 'docs');
  if (existsSync(docs) && statSync(docs).isDirectory()) walk(docs, root, found);
  for (const entry of readdirSync(root, { withFileTypes: true })) {
    if (entry.isFile() && entry.name.endsWith('.md')) found.push(entry.name);
  }
  return found.filter((file) => !EXCLUDED.some((pattern) => pattern.test(file))).sort();
}

function walk(absoluteDir, root, found) {
  for (const entry of readdirSync(absoluteDir, { withFileTypes: true })) {
    const absolute = path.join(absoluteDir, entry.name);
    if (entry.isDirectory()) {
      if (!IGNORED_DIRS.has(entry.name)) walk(absolute, root, found);
      continue;
    }
    if (entry.isFile() && entry.name.endsWith('.md')) {
      found.push(toPosix(path.relative(root, absolute)));
    }
  }
  return found;
}

/**
 * Las lineas fuera de los bloques de codigo, numeradas desde 1. Un enlace dentro
 * de un ``` es un ejemplo y no se comprueba.
 */
function proseLines(text) {
  const lines = [];
  let fenced = false;
  for (const [index, content] of text.split('\n').entries()) {
    if (/^\s*(```|~~~)/u.test(content)) {
      fenced = !fenced;
      continue;
    }
    if (!fenced) lines.push({ content, line: index + 1 });
  }
  return lines;
}

/**
 * El ancla que GitHub genera para un titulo: minusculas, sin el formato de
 * Markdown, los espacios a guiones y fuera todo lo que no sea letra, numero,
 * guion o subrayado. Los acentos se conservan, como hace GitHub.
 */
export function slug(heading) {
  return heading
    .trim()
    .toLowerCase()
    .replace(/`([^`]*)`/gu, '$1')
    .replace(/!?\[([^\]]*)\]\([^()\s]*\)/gu, '$1')
    .replace(/[*_~]/gu, '')
    .replace(/[^\p{L}\p{N}\s_-]/gu, '')
    .trim()
    .replace(/\s+/gu, '-');
}

/** Las anclas de un documento: una por titulo, con el sufijo de GitHub si repite. */
function anchorsOf(text) {
  const seen = new Map();
  const anchors = new Set();
  for (const { content } of proseLines(text)) {
    const heading = /^#{1,6}\s+(.*)$/u.exec(content)?.[1];
    if (heading === undefined) continue;
    const base = slug(heading);
    if (base === '') continue;
    const count = seen.get(base) ?? 0;
    seen.set(base, count + 1);
    anchors.add(count === 0 ? base : `${base}-${String(count)}`);
  }
  return anchors;
}

function isExternal(target) {
  return /^(?:[a-z][a-z0-9+.-]*:|\/\/)/iu.test(target) || target.startsWith('mailto:');
}

/** Resuelve el destino de un enlace relativo a rutas del repositorio. */
function resolveTarget(file, target) {
  const [rawPath, anchor] = splitAnchor(target);
  if (rawPath === '') return { file, anchor };
  const decoded = decodeURIComponent(rawPath);
  return { file: toPosix(path.join(path.dirname(file), decoded)), anchor };
}

function splitAnchor(target) {
  const hash = target.indexOf('#');
  return hash === -1 ? [target, undefined] : [target.slice(0, hash), target.slice(hash + 1)];
}

/** Las anclas de cada documento, leidas una sola vez. */
function anchorIndex(root) {
  const cache = new Map();
  return (file) => {
    if (!cache.has(file)) {
      const absolute = path.join(root, file);
      const text = existsSync(absolute) ? readFileSync(absolute, 'utf8') : null;
      cache.set(file, text === null ? null : anchorsOf(text));
    }
    return cache.get(file);
  };
}

function adrNumbers(root) {
  const dir = path.join(root, 'docs', 'adr');
  if (!existsSync(dir)) return new Set();
  const numbers = new Set();
  for (const name of readdirSync(dir)) {
    const match = /^(\d{4})-.*\.md$/u.exec(name);
    if (match?.[1] !== undefined) numbers.add(match[1]);
  }
  return numbers;
}

function linkViolations(file, lines, anchorsFor, root) {
  const violations = [];
  for (const { content, line } of lines) {
    for (const match of content.matchAll(LINK)) {
      const target = match[1] ?? '';
      if (isExternal(target)) continue;
      violations.push(...checkTarget({ file, line, target, anchorsFor, root }));
    }
  }
  return violations;
}

function checkTarget({ file, line, target, anchorsFor, root }) {
  const resolved = resolveTarget(file, target);
  if (!existsSync(path.join(root, resolved.file))) {
    return [{ file, line, message: `enlace roto: ${target} (no existe ${resolved.file})` }];
  }
  if (resolved.anchor === undefined || !resolved.file.endsWith('.md')) return [];
  const anchors = anchorsFor(resolved.file);
  if (anchors === null || anchors.has(resolved.anchor)) return [];
  return [
    { file, line, message: `ancla inexistente: ${target} (${resolved.file} no tiene ese titulo)` },
  ];
}

function adrViolations(file, lines, known) {
  const violations = [];
  for (const { content, line } of lines) {
    for (const match of content.matchAll(ADR_MENTION)) {
      const number = match[1] ?? '';
      if (known.has(number)) continue;
      violations.push({ file, line, message: `ADR ${number} citado y no existe en docs/adr/` });
    }
  }
  return violations;
}

export function checkDocsLinks({ root }) {
  const files = markdownFiles(root);
  const anchorsFor = anchorIndex(root);
  const known = adrNumbers(root);
  const violations = [];
  for (const file of files) {
    const lines = proseLines(readFileSync(path.join(root, file), 'utf8'));
    violations.push(...linkViolations(file, lines, anchorsFor, root));
    violations.push(...adrViolations(file, lines, known));
  }
  return { scanned: files.length, violations };
}

if (import.meta.main) {
  run((options) => report({ name: 'enlaces y anclas de docs', ...checkDocsLinks(options) }));
}
