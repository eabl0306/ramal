import { type QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { type ReactElement, type ReactNode, useState } from 'react';

import { createRepoViewQueryClient } from '../data/query-client';

export interface RepoViewProviderProps {
  /**
   * QueryClient del host, si ya tiene uno. Sin el, el proveedor crea el suyo una
   * vez con createRepoViewQueryClient y lo mantiene mientras viva.
   */
  readonly queryClient?: QueryClient;
  readonly children: ReactNode;
}

/**
 * Lo que necesitan los hooks de datos de @ramal/repo-view: un QueryClient. Lo
 * pone cada host en su composition root, junto a I18nProvider e IconProvider.
 */
export function RepoViewProvider({ queryClient, children }: RepoViewProviderProps): ReactElement {
  const [ownClient] = useState(() => queryClient ?? createRepoViewQueryClient());
  return <QueryClientProvider client={queryClient ?? ownClient}>{children}</QueryClientProvider>;
}
