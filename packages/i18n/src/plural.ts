/**
 * Elige la forma plural de una clave con Intl.PluralRules sobre `count`.
 *
 * Convencion de datos de i18next: `clave_one`, `clave_other` y, si hace falta,
 * `clave_zero`. Si el idioma pide una categoria que el diccionario no tiene (el
 * castellano tiene `many` para los millones), se usa `_other`.
 */
export function pluralTemplate(
  messages: ReadonlyMap<string, string>,
  key: string,
  count: number,
  rules: Intl.PluralRules,
): string | undefined {
  if (count === 0) {
    const zero = messages.get(`${key}_zero`);
    if (zero !== undefined) return zero;
  }
  return messages.get(`${key}_${rules.select(count)}`) ?? messages.get(`${key}_other`);
}
