import { readFileSync } from 'node:fs';
import path from 'node:path';

export function readLines(root, relativePath) {
  return readFileSync(path.join(root, relativePath), 'utf8').split('\n');
}

/** Un comentario en la misma linea o en la anterior cuenta como justificacion. */
export function hasNearbyComment(lines, index, marker) {
  const current = lines[index] ?? '';
  const afterMarker = current.slice(current.indexOf(marker) + marker.length);
  if (/\/\/\s*\S/u.test(afterMarker)) return true;
  const previous = (lines[index - 1] ?? '').trim();
  return /^(\/\/|\/\*|\*)\s*\S/u.test(previous);
}
