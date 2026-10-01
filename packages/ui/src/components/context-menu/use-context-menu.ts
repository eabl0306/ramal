import { type MouseEvent, useCallback, useMemo, useState } from 'react';

import type { MenuAnchor } from './menu-model';

export interface ContextMenuController {
  readonly anchor: MenuAnchor | null;
  /** Para onContextMenu: abre en el puntero y evita el menu del navegador. */
  readonly openAtPointer: (event: MouseEvent) => void;
  /** Para el boton ⋯: abre pegado al boton. */
  readonly openAtElement: (element: Element) => void;
  readonly close: () => void;
}

/**
 * Estado de un menu contextual. La tecla de menu del teclado (o Mayus+F10) lanza
 * un contextmenu sin coordenadas; entonces el menu se abre pegado al elemento,
 * que es lo que espera quien no usa raton.
 */
export function useContextMenu(): ContextMenuController {
  const [anchor, setAnchor] = useState<MenuAnchor | null>(null);
  const openAtElement = useCallback((element: Element) => {
    setAnchor({ kind: 'element', element });
  }, []);
  const openAtPointer = useCallback((event: MouseEvent) => {
    event.preventDefault();
    if (event.clientX === 0 && event.clientY === 0) {
      setAnchor({ kind: 'element', element: event.currentTarget });
      return;
    }
    setAnchor({ kind: 'point', x: event.clientX, y: event.clientY });
  }, []);
  const close = useCallback(() => {
    setAnchor(null);
  }, []);
  return useMemo(
    () => ({ anchor, openAtPointer, openAtElement, close }),
    [anchor, openAtPointer, openAtElement, close],
  );
}
