import type { Locale } from '@git-rudder/i18n';
import { I18nProvider, IconProvider, type IconSet } from '@git-rudder/ui';
import { lucideIcons } from '@git-rudder/ui/icons/lucide';
import { QueryClient } from '@tanstack/react-query';
import { type RenderResult, render } from '@testing-library/react';
import type { ReactElement } from 'react';

import { RepoViewProvider } from '../view/repo-view-provider';

export interface RenderOptions {
  readonly locale?: Locale;
  readonly icons?: IconSet;
  readonly queryClient?: QueryClient;
}

/** QueryClient de test: sin reintentos ni recogida de basura que deje temporizadores vivos. */
export function testQueryClient(): QueryClient {
  return new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: Number.POSITIVE_INFINITY } },
  });
}

/** Monta con los mismos proveedores que pone cada host en su composition root. */
export function renderRepoView(
  ui: ReactElement,
  { locale = 'es', icons = lucideIcons, queryClient = testQueryClient() }: RenderOptions = {},
): RenderResult & { readonly queryClient: QueryClient } {
  const result = render(
    <I18nProvider locale={locale}>
      <IconProvider icons={icons}>
        <RepoViewProvider queryClient={queryClient}>{ui}</RepoViewProvider>
      </IconProvider>
    </I18nProvider>,
  );
  return { ...result, queryClient };
}
