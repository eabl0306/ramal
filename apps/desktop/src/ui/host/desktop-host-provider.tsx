import type { ReactElement, ReactNode } from 'react';

import { type DesktopHost, DesktopHostContext } from './desktop-host-context';

export interface DesktopHostProviderProps {
  readonly host: DesktopHost;
  readonly children: ReactNode;
}

/** Lo pone el composition root con los adaptadores de Tauri (o los de un test). */
export function DesktopHostProvider({ host, children }: DesktopHostProviderProps): ReactElement {
  return <DesktopHostContext value={host}>{children}</DesktopHostContext>;
}
