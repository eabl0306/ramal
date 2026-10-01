import type { GraphRow } from '@ramal/bridge';
import { type KeyboardEvent, type MouseEvent, useCallback, useMemo, useState } from 'react';

import { nextRowIndex } from '../graph/keyboard';

export interface SelectionOptions {
  readonly rows: readonly GraphRow[];
  /**
   * Sha seleccionado si el host controla la seleccion; `undefined` deja que la
   * tabla la lleve sola.
   */
  readonly selectedSha: string | null | undefined;
  readonly onSelectionChange: ((row: GraphRow) => void) | undefined;
}

export interface GridSelection {
  readonly selectedIndex: number | null;
  readonly select: (index: number) => void;
}

/**
 * Seleccion por sha y no por indice: al llegar paginas nuevas los indices de lo
 * ya cargado no cambian, pero el dia que el motor avise de cambios (planificado)
 * el commit seleccionado puede moverse de fila y debe seguir seleccionado.
 */
export function useGridSelection({
  rows,
  selectedSha,
  onSelectionChange,
}: SelectionOptions): GridSelection {
  const [ownSha, setOwnSha] = useState<string | null>(null);
  const controlled = selectedSha !== undefined;
  const currentSha = controlled ? selectedSha : ownSha;
  const selectedIndex = useMemo(() => {
    if (currentSha === null) return null;
    const index = rows.findIndex((row) => row.sha === currentSha);
    return index === -1 ? null : index;
  }, [rows, currentSha]);
  const select = useCallback(
    (index: number) => {
      const row = rows[index];
      if (row === undefined) return;
      if (!controlled) setOwnSha(row.sha);
      onSelectionChange?.(row);
    },
    [rows, controlled, onSelectionChange],
  );
  return { selectedIndex, select };
}

export interface GridInputOptions {
  readonly rows: readonly GraphRow[];
  readonly selection: GridSelection;
  readonly onActivate: ((row: GraphRow) => void) | undefined;
  /** Desplaza lo justo para que la fila se vea. */
  readonly scrollToIndex: (index: number) => void;
  /** Filas que caben en pantalla, para Re Pag y Av Pag. */
  readonly pageRows: () => number;
}

export interface GridInput {
  readonly onKeyDown: (event: KeyboardEvent<HTMLElement>) => void;
  readonly onClick: (event: MouseEvent<HTMLElement>) => void;
  readonly onDoubleClick: (event: MouseEvent<HTMLElement>) => void;
}

/**
 * Teclado y raton de la tabla entera. El foco esta en la tabla (una sola parada
 * de tabulador) y la fila activa se anuncia con aria-activedescendant: con filas
 * virtualizadas, mover el foco a una fila que puede desmontarse lo perderia.
 * Flechas, Re Pag, Av Pag, Inicio y Fin mueven la seleccion; Intro activa la
 * fila, igual que el doble clic.
 */
export function useGridInput({
  rows,
  selection,
  onActivate,
  scrollToIndex,
  pageRows,
}: GridInputOptions): GridInput {
  const { selectedIndex, select } = selection;
  const activate = useCallback(
    (index: number | null) => {
      const row = index === null ? undefined : rows[index];
      if (row !== undefined) onActivate?.(row);
    },
    [rows, onActivate],
  );
  const onKeyDown = useCallback(
    (event: KeyboardEvent<HTMLElement>) => {
      // Las teclas dentro de un boton de la tabla (reintentar) son suyas.
      if (event.target !== event.currentTarget) return;
      if (event.key === 'Enter') {
        event.preventDefault();
        activate(selectedIndex);
        return;
      }
      const next = nextRowIndex(event.key, selectedIndex, rows.length, pageRows());
      if (next === null) return;
      event.preventDefault();
      select(next);
      scrollToIndex(next);
    },
    [activate, pageRows, rows.length, scrollToIndex, select, selectedIndex],
  );
  const onClick = useCallback(
    (event: MouseEvent<HTMLElement>) => {
      const index = rowIndexOf(event.target);
      if (index !== null) select(index);
    },
    [select],
  );
  const onDoubleClick = useCallback(
    (event: MouseEvent<HTMLElement>) => {
      activate(rowIndexOf(event.target));
    },
    [activate],
  );
  return { onKeyDown, onClick, onDoubleClick };
}

/** Indice de la fila sobre la que cayo un clic, por su `data-row-index`. */
function rowIndexOf(target: EventTarget): number | null {
  if (!(target instanceof Element)) return null;
  const raw = target.closest<HTMLElement>('[data-row-index]')?.dataset['rowIndex'];
  return raw === undefined ? null : Number(raw);
}
