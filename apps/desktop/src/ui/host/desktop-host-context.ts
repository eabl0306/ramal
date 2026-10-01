import type { BridgeClient } from '@ramal/bridge';
import { createContext, use } from 'react';

import type { RepositoryPicker } from '../../application/ports/repository-picker';

/**
 * Las capacidades del host que usan las pantallas: el cliente del motor (para la
 * vista de repositorio) y el selector de repositorios. Las construye el composition
 * root; la interfaz solo las recibe por contexto, nunca importa un adaptador.
 */
export interface DesktopHost {
  readonly client: BridgeClient;
  readonly picker: RepositoryPicker;
}

export const DesktopHostContext = createContext<DesktopHost | null>(null);

export function useDesktopHost(): DesktopHost {
  const host = use(DesktopHostContext);
  if (host === null) {
    throw new Error('useDesktopHost necesita un <DesktopHostProvider> por encima en el arbol.');
  }
  return host;
}
