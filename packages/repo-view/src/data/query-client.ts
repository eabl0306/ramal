import { QueryClient } from '@tanstack/react-query';

/**
 * QueryClient con los valores que tienen sentido para hablar con el motor local:
 * sin refrescos por foco ni por red (el motor no esta en la red, y en un webview
 * el foco cambia sin que cambie nada) y sin reintentos por defecto; cada consulta
 * de @git-rudder/repo-view decide los suyos segun el tipo de error.
 *
 * Un host que ya tenga su propio QueryClient puede pasarselo a RepoViewProvider
 * en lugar de este.
 */
export function createRepoViewQueryClient(): QueryClient {
  return new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
        refetchOnWindowFocus: false,
        refetchOnReconnect: false,
      },
    },
  });
}
