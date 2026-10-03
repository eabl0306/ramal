import { RepoViewProvider } from '@git-rudder/repo-view';
import { IconProvider } from '@git-rudder/ui';
import { lucideIcons } from '@git-rudder/ui/icons/lucide';
import { type ReactElement, useMemo } from 'react';

import { AppearanceProvider } from '../ui/appearance/appearance-provider';
import type { DesktopHost } from '../ui/host/desktop-host-context';
import { DesktopHostProvider } from '../ui/host/desktop-host-provider';
import { AppShell } from '../ui/shell/app-shell';

import type { Container } from './composition-root';

export interface AppProps {
  readonly container: Container;
  /** El elemento que recibe `data-theme` y `lang`: `<html>` en la app. */
  readonly root: HTMLElement;
}

/**
 * Los proveedores que pone el escritorio (architecture.md 7.4): tema de Git Rudder e
 * idioma elegidos por el usuario, iconos de Lucide, el QueryClient de la vista de
 * repositorio y las capacidades del host. La densidad comoda va en `<html>`.
 */
export function App({ container, root }: AppProps): ReactElement {
  const host = useMemo<DesktopHost>(
    () => ({ client: container.client, picker: container.picker }),
    [container],
  );
  return (
    <AppearanceProvider
      initial={container.initialAppearance}
      store={container.appearanceStore}
      root={root}
    >
      <IconProvider icons={lucideIcons}>
        <RepoViewProvider queryClient={container.queryClient}>
          <DesktopHostProvider host={host}>
            <AppShell />
          </DesktopHostProvider>
        </RepoViewProvider>
      </IconProvider>
    </AppearanceProvider>
  );
}
