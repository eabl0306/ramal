/** Los dos temas de `@ramal/ui/themes/ramal.css`, que se ponen en `<html data-theme>`. */
export const THEMES = ['dark', 'light'] as const;

export type ThemeName = (typeof THEMES)[number];

export function isThemeName(value: unknown): value is ThemeName {
  return value === 'dark' || value === 'light';
}

/**
 * El ajuste de apariencia guardado en este equipo. Cada campo falta hasta que el
 * usuario lo elige por primera vez. El idioma va como texto: decidir que idiomas
 * existen es cosa de @ramal/i18n, que esta capa no conoce.
 */
export interface StoredAppearance {
  readonly theme?: ThemeName;
  readonly locale?: string;
}

/**
 * Donde vive la apariencia entre arranques. Es de este dispositivo; la
 * sincronizacion con la cuenta de Ramal esta planificada.
 */
export interface AppearanceStore {
  load(): StoredAppearance;
  save(appearance: Required<StoredAppearance>): void;
}
