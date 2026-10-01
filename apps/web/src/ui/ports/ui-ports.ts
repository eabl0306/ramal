import { createContext, use } from 'react';

import type { AuthGateway } from '../../application/ports/auth-gateway';
import type { Clipboard } from '../../application/ports/clipboard';
import type { PreferencesStore } from '../../application/ports/preferences-store';
import type { VisitorEnvironment } from '../../application/ports/visitor-environment';

/**
 * Los puertos que la interfaz recibe ya montados. Las pantallas nunca
 * construyen adaptadores: los pone el composition root con UiPortsContext, y
 * los tests de componente con los de memoria.
 */
export interface UiPorts {
  readonly auth: AuthGateway;
  readonly preferences: PreferencesStore;
  readonly visitor: VisitorEnvironment;
  readonly clipboard: Clipboard;
}

export const UiPortsContext = createContext<UiPorts | null>(null);

export function useUiPorts(): UiPorts {
  const ports = use(UiPortsContext);
  if (ports === null) {
    throw new Error('Falta UiPortsContext por encima en el arbol: lo pone src/app/root.tsx.');
  }
  return ports;
}
