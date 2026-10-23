/**
 * Color del avatar de una persona, derivado de su nombre.
 *
 * Es un dato que se calcula en el cliente y no se le pide a la API (gaps B-34):
 * el mismo nombre tiene que dar siempre el mismo color, en los tres productos y
 * sin preguntar a nadie.
 *
 * Aqui solo sale el **indice** dentro de la paleta, no el color: `@git-rudder/domain`
 * no sabe de CSS ni de temas. Quien pinta lo traduce a su token (`laneColor` en
 * `@git-rudder/ui`), que es la misma paleta de carriles del grafo, para que la
 * interfaz no invente un segundo juego de colores (gaps B-14).
 */

/**
 * Huella del nombre. FNV-1a de 32 bits sobre los puntos de codigo, que es corto,
 * estable y reparte bien nombres parecidos ("Ana Ruiz" y "Ana Rubio" no caen en
 * el mismo color por empezar igual).
 *
 * Se recorre por puntos de codigo (`for...of`) y no por unidades UTF-16, para que
 * un nombre con un emoji o con un caracter fuera del plano basico de una huella
 * y no dos mitades.
 */
const FNV_OFFSET = 2_166_136_261;
const FNV_PRIME = 16_777_619;

function fingerprint(name: string): number {
  let hash = FNV_OFFSET;
  for (const char of name) {
    hash ^= char.codePointAt(0) ?? 0;
    // Math.imul mantiene la multiplicacion en 32 bits; sin el, el numero pasa de
    // los enteros seguros de JavaScript y la huella deja de ser estable.
    hash = Math.imul(hash, FNV_PRIME);
  }
  return hash >>> 0;
}

/**
 * Indice en una paleta de `paletteSize` colores. El modulo es lo que evita que un
 * nombre se quede sin color cuando la paleta es mas corta que los nombres
 * posibles, que es el fallo que tenia el prototipo con los carriles (gaps B-14).
 *
 * El nombre se normaliza igual que para las iniciales: se recorta y se pasa a
 * minusculas con `toLowerCase` (nunca `toLocaleLowerCase`, que con la i turca
 * daria un color distinto segun la maquina). Un nombre vacio da 0: es un color
 * valido, y asi no hay que tratar el caso en cada sitio que pinta.
 */
export function avatarColorIndex(name: string, paletteSize: number): number {
  const size = Math.trunc(paletteSize);
  if (size <= 0) return 0;
  const clean = name.trim().toLowerCase();
  if (clean === '') return 0;
  return fingerprint(clean) % size;
}
