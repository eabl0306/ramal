/**
 * Acceso a las clases de un CSS Module. Con `noUncheckedIndexedAccess` cada
 * indice puede ser undefined; si una clase no existiera, el DOM llevaria su
 * propio nombre, que es mas facil de encontrar al depurar que una clase vacia.
 * Las clases van en kebab-case, como pide stylelint, y se leen por su nombre.
 */
export function moduleClasses(styles: Readonly<Record<string, string>>): (name: string) => string {
  return (name) => styles[name] ?? name;
}

/** Nombre de producto: es una marca y no se traduce. */
export const PRODUCT_NAME = 'Git Rudder';
