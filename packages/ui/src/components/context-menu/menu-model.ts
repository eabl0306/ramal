import type { IconName } from '../../icons/icon-names';

/**
 * Los menus contextuales se describen como datos: asi el mismo menu se abre con
 * clic derecho o con el boton ⋯, y repo-view puede construirlos sin conocer
 * React Aria.
 */
export interface MenuActionEntry {
  readonly kind?: 'item';
  readonly id: string;
  readonly label: string;
  /** Atajo ya formateado con formatShortcut de @ramal/i18n. */
  readonly hint?: string;
  readonly icon?: IconName;
  /** Acciones destructivas, en el color de peligro. */
  readonly danger?: boolean;
  readonly disabled?: boolean;
  /**
   * Casilla marcable (workspaces de un repo, filtro de plataformas). Con valor,
   * el elemento se anuncia como casilla y pinta la marca en el componente.
   */
  readonly checked?: boolean;
  /** No cierra el menu al elegirlo: permite marcar varias casillas seguidas. */
  readonly keepOpen?: boolean;
  readonly onAction?: () => void;
  /** Submenu que se abre al pasar por encima o con la flecha derecha. */
  readonly children?: readonly MenuEntry[];
}

export interface MenuSeparatorEntry {
  readonly kind: 'separator';
  readonly id: string;
}

export type MenuEntry = MenuActionEntry | MenuSeparatorEntry;

/** Donde se abre: en el puntero (clic derecho) o pegado a un elemento (boton ⋯). */
export type MenuAnchor =
  | { readonly kind: 'point'; readonly x: number; readonly y: number }
  | { readonly kind: 'element'; readonly element: Element };

export type EntryGroup =
  | { readonly kind: 'separator'; readonly id: string }
  | { readonly kind: 'item'; readonly entry: MenuActionEntry }
  | { readonly kind: 'checks'; readonly id: string; readonly entries: readonly MenuActionEntry[] };

function isCheckable(entry: MenuEntry): entry is MenuActionEntry {
  return entry.kind !== 'separator' && entry.checked !== undefined;
}

/**
 * Junta las casillas consecutivas en un grupo: React Aria da la semantica de
 * casilla por seccion, no por elemento suelto.
 */
export function groupEntries(entries: readonly MenuEntry[]): EntryGroup[] {
  const groups: EntryGroup[] = [];
  for (const entry of entries) {
    const last = groups.at(-1);
    if (entry.kind === 'separator') {
      groups.push({ kind: 'separator', id: entry.id });
    } else if (!isCheckable(entry)) {
      groups.push({ kind: 'item', entry });
    } else if (last?.kind === 'checks') {
      groups[groups.length - 1] = { ...last, entries: [...last.entries, entry] };
    } else {
      groups.push({ kind: 'checks', id: `checks-${entry.id}`, entries: [entry] });
    }
  }
  return groups;
}

/** La casilla que acaba de cambiar al comparar la seleccion nueva con la marcada. */
export function toggledEntry(
  entries: readonly MenuActionEntry[],
  selected: ReadonlySet<string | number>,
): MenuActionEntry | undefined {
  return entries.find((entry) => selected.has(entry.id) !== (entry.checked === true));
}
