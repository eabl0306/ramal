import { I18nProvider, IconProvider, ToastProvider } from '@git-rudder/ui';
import { lucideIcons } from '@git-rudder/ui/icons/lucide';
import type { ReactElement, ReactNode } from 'react';

import type { Language } from '../domain/preferences/preferences';
import { type UiPorts, UiPortsContext } from '../ui/ports/ui-ports';

export interface AppProvidersProps {
  readonly ports: UiPorts;
  readonly language: Language;
  readonly children: ReactNode;
}

/** Textos, iconos de Lucide, avisos y los puertos: lo que la web pone sobre toda pantalla. */
export function AppProviders({ ports, language, children }: AppProvidersProps): ReactElement {
  return (
    <I18nProvider locale={language}>
      <IconProvider icons={lucideIcons}>
        <ToastProvider>
          <UiPortsContext value={ports}>{children}</UiPortsContext>
        </ToastProvider>
      </IconProvider>
    </I18nProvider>
  );
}
