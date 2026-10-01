/**
 * Iniciales del autor para el nodo del grafo y el avatar de la columna de autor.
 * El prototipo las traia escritas en los datos (`au: 'MR'`); el motor da el
 * nombre tal como esta en el commit, asi que se calculan aqui.
 *
 * - Dos palabras o mas: primera letra de la primera y de la ultima ("Ana Maria
 *   Ruiz" da "AR").
 * - Una palabra: sus dos primeras letras ("ezequiel" da "EZ").
 * - Sin letras ni numeros: "?".
 *
 * Se trabaja con puntos de codigo (Array.from) para no partir un caracter fuera
 * del plano basico, y los separadores habituales de un nombre de usuario (punto,
 * guion, guion bajo, arroba) cuentan como espacios.
 */
const SEPARATORS = /[\s._@-]+/u;
const ALPHANUMERIC = /[\p{L}\p{N}]/u;

export function authorInitials(name: string): string {
  const words = name
    .split(SEPARATORS)
    .map((word) => Array.from(word).filter((char) => ALPHANUMERIC.test(char)))
    .filter((chars) => chars.length > 0);
  const first = words[0];
  if (first === undefined) return '?';
  const last = words.at(-1) ?? first;
  const letters = words.length > 1 ? [first[0], last[0]] : first.slice(0, 2);
  // toUpperCase y no toLocaleUpperCase: el resultado no puede depender del idioma
  // de la maquina (la i turca).
  return letters.join('').toUpperCase();
}
