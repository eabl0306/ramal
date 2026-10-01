/**
 * Los tokens semanticos que leen los componentes (architecture.md 7.1). Los dos
 * temas dan valor a todos; un test lo comprueba leyendo ramal.css y vscode.css.
 *
 * Desde TypeScript solo se usan como var(--...): un color literal en un
 * componente lo rechaza ramal/no-raw-colors.
 */
export const COLOR_TOKENS = [
  '--r-bg',
  '--r-panel',
  '--r-surface',
  '--r-surface-hover',
  '--r-border',
  '--r-text',
  '--r-muted',
  '--r-faint',
  '--r-accent',
  '--r-accent-ink',
  '--r-accent-soft',
  '--r-ok',
  '--r-warn',
  '--r-danger',
  '--r-info',
  '--r-merged',
  '--r-add-bg',
  '--r-del-bg',
  '--r-status-m',
  '--r-status-a',
  '--r-status-d',
  '--r-status-r',
  '--r-status-c',
  '--r-focus',
  '--r-backdrop',
  '--r-knob',
] as const;

export const SHAPE_TOKENS = [
  '--r-font-ui',
  '--r-font-code',
  '--r-font-size-base',
  '--r-radius-sm',
  '--r-radius-md',
  '--r-radius-lg',
  '--r-radius-xl',
  '--r-shadow-overlay',
  '--r-hatch',
] as const;

/** Iguales en todos los temas: viven en base.css. */
export const SHARED_TOKENS = [
  '--r-lane-0',
  '--r-lane-1',
  '--r-lane-2',
  '--r-lane-3',
  '--r-lane-4',
  '--r-lane-5',
  '--r-on-lane',
  '--r-on-emphasis',
  '--r-platform-github',
  '--r-platform-gitlab',
  '--r-platform-bitbucket',
  '--r-platform-gitea',
  '--r-platform-azure',
  '--r-provider-google',
  '--r-ok-soft',
  '--r-warn-soft',
  '--r-danger-soft',
  '--r-info-soft',
  '--r-merged-soft',
  '--r-faint-soft',
  '--r-hc-border',
  '--r-hc-active-border',
  '--r-row-h',
  '--r-menu-item-h',
  '--r-sidebar-item-h',
  '--r-toolbar-h',
] as const;

export type ThemeToken =
  (typeof COLOR_TOKENS)[number] | (typeof SHAPE_TOKENS)[number] | (typeof SHARED_TOKENS)[number];

/** `var(--r-...)` listo para una prop `style` o un atributo SVG. */
export function cssVar(token: ThemeToken): string {
  return `var(${token})`;
}

export const LANE_COUNT = 6;

/**
 * Color del carril de un commit. Con el modulo, a partir del carril 6 vuelve a
 * empezar la paleta (gaps B-14: el prototipo se quedaba sin color).
 */
export function laneColor(lane: number): string {
  const index = ((Math.trunc(lane) % LANE_COUNT) + LANE_COUNT) % LANE_COUNT;
  return `var(--r-lane-${String(index)})`;
}
