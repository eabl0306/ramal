import type { CSSProperties } from 'react';

/**
 * Acceso a las clases de un CSS Module. Con `noUncheckedIndexedAccess` cada
 * indice puede ser undefined; si una clase no existiera, el DOM llevaria su
 * propio nombre, que se encuentra antes al depurar que una clase vacia.
 */
export function moduleClasses(styles: Readonly<Record<string, string>>): (name: string) => string {
  return (name) => styles[name] ?? name;
}

/**
 * Variables CSS propias en una prop `style`, con el nombre obligado a empezar
 * por `--`. React las aplica por CSSOM, que la CSP de los webviews no bloquea
 * (architecture.md 6.8).
 */
export function cssVars(vars: Readonly<Record<`--${string}`, string>>): CSSProperties {
  return vars;
}
