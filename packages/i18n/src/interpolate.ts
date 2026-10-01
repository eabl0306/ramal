/**
 * Sustituye los marcadores {nombre} por sus valores. Los numeros se formatean
 * con el idioma de la interfaz ("12.345" en castellano); los textos van tal cual.
 * Un marcador sin valor se deja a la vista: es un fallo que el tipado ya impide
 * y que asi se ve en pantalla en vez de esconderse.
 */

const PLACEHOLDER = /\{([A-Za-z][A-Za-z0-9]*)\}/gu;

export type InterpolationValues = Readonly<Record<string, string | number>>;

export function interpolate(
  template: string,
  values: InterpolationValues | undefined,
  formatNumber: (value: number) => string,
): string {
  if (values === undefined) return template;
  return template.replace(PLACEHOLDER, (match, name: string) => {
    const value = values[name];
    if (value === undefined) return match;
    return typeof value === 'number' ? formatNumber(value) : value;
  });
}

/** Los nombres de los marcadores de un texto, en orden y sin repetir. */
export function placeholdersOf(template: string): string[] {
  return [...new Set([...template.matchAll(PLACEHOLDER)].map((match) => match[1] ?? ''))];
}
