/**
 * Enmascarado de llaves y tokens para ensenarlos sin revelarlos.
 *
 * El prototipo ensenaba entera cualquier llave de menos de 11 caracteres,
 * porque el prefijo y el sufijo se solapaban, y usaba prefijos de 7 o de 6
 * segun el producto (gaps B-34). Aqui: por debajo de 12 caracteres, mascara
 * completa; por encima, el prefijo conocido del proveedor (que no es secreto)
 * mas los 4 ultimos, y siempre al menos 8 caracteres ocultos.
 *
 * En produccion la mascara la calcula el motor o la API y la llave en claro no
 * vuelve a la interfaz; esta funcion es la misma regla para quien la necesite.
 */

export const SECRET_MASK = '••••••••';

/** Por debajo de esto no se ensena nada. */
export const MIN_MASKABLE_LENGTH = 12;

const VISIBLE_SUFFIX = 4;
const MIN_HIDDEN = 8;

/** Prefijos publicos de los proveedores, el mas largo primero para que gane el mas especifico. */
const KNOWN_PREFIXES = [
  'github_pat_',
  'sk-proj-',
  'sk-ant-',
  'sk-or-',
  'glpat-',
  'ghp_',
  'gsk_',
  'AIza',
  'sk-',
];

function visiblePrefix(secret: string): string {
  const prefix = KNOWN_PREFIXES.find((candidate) => secret.startsWith(candidate)) ?? '';
  const hidden = secret.length - prefix.length - VISIBLE_SUFFIX;
  return hidden >= MIN_HIDDEN ? prefix : '';
}

/** Devuelve '' si no hay llave: la interfaz pinta su guion de "sin llave". */
export function maskSecret(raw: string): string {
  const secret = raw.trim();
  if (secret === '') return '';
  if (secret.length < MIN_MASKABLE_LENGTH) return SECRET_MASK;
  return `${visiblePrefix(secret)}${SECRET_MASK}${secret.slice(-VISIBLE_SUFFIX)}`;
}
