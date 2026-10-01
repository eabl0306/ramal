import type { BridgeClient, GraphRow } from '@ramal/bridge';
import { useInfiniteQuery } from '@tanstack/react-query';
import { useCallback, useMemo } from 'react';

import {
  DEFAULT_PAGE_SIZE,
  graphWindowKey,
  mergePages,
  nextOffset,
  shouldRetry,
} from './graph-window-query';

export interface UseGraphWindowOptions {
  /** Filas por peticion a `graph/window`, de 1 a 1000. */
  readonly pageSize?: number;
}

/** Lo que la vista necesita de la ventana del grafo, sin tipos de TanStack. */
export interface GraphWindowState {
  /** `pending` hasta la primera pagina; `error` si la primera pagina fallo. */
  readonly status: 'pending' | 'error' | 'success';
  /** Filas cargadas hasta ahora; el indice es la fila absoluta del motor. */
  readonly rows: readonly GraphRow[];
  readonly laneCount: number;
  /** Quedan commits por pedir. */
  readonly hasMore: boolean;
  /** Error de la primera pagina, o `null`. */
  readonly error: unknown;
  readonly isLoadingMore: boolean;
  /** Error de la ultima pagina pedida al bajar, o `null`. */
  readonly loadMoreError: unknown;
  /** Pide la pagina siguiente si hay y no se esta pidiendo ya. */
  readonly loadMore: () => void;
  /** Vuelve a intentar lo que fallo: la primera pagina o la siguiente. */
  readonly retry: () => void;
}

/**
 * La ventana del grafo de un repositorio, paginada al bajar. Necesita un
 * QueryClientProvider por encima (RepoViewProvider lo pone).
 *
 * El grafo no cambia mientras el motor no avise, asi que no se refresca al
 * volver el foco ni al reconectar: en un webview de VS Code el foco va y viene
 * sin que el repositorio cambie, y refrescar volveria a pedir todas las paginas.
 * La cancelacion de TanStack llega al motor como `$/cancel` por el `signal`.
 */
export function useGraphWindow(
  client: BridgeClient,
  repoId: string,
  options: UseGraphWindowOptions = {},
): GraphWindowState {
  const pageSize = options.pageSize ?? DEFAULT_PAGE_SIZE;
  const query = useInfiniteQuery({
    queryKey: graphWindowKey(repoId, pageSize),
    queryFn: ({ pageParam, signal }) =>
      client.call('graph/window', { repoId, offset: pageParam, limit: pageSize }, { signal }),
    initialPageParam: 0,
    getNextPageParam: nextOffset,
    retry: shouldRetry,
    staleTime: Number.POSITIVE_INFINITY,
    refetchOnWindowFocus: false,
    refetchOnReconnect: false,
  });
  const { data, error, fetchNextPage, refetch, hasNextPage, isFetchingNextPage } = query;
  const loaded = useMemo(() => mergePages(data?.pages ?? []), [data]);
  const loadMoreError = data !== undefined && query.isFetchNextPageError ? error : null;

  const loadMore = useCallback(() => {
    if (hasNextPage && !isFetchingNextPage) void fetchNextPage();
  }, [fetchNextPage, hasNextPage, isFetchingNextPage]);

  const retry = useCallback(() => {
    if (data === undefined) {
      void refetch();
      return;
    }
    void fetchNextPage();
  }, [data, fetchNextPage, refetch]);

  return {
    status: statusOf(data !== undefined, query.isError),
    rows: loaded.rows,
    laneCount: loaded.laneCount,
    hasMore: hasNextPage,
    error: data === undefined ? error : null,
    isLoadingMore: isFetchingNextPage,
    loadMoreError,
    loadMore,
    retry,
  };
}

/**
 * En TanStack Query 5 una pagina siguiente que falla tambien pone la consulta
 * en `error`. Para la vista eso no es un error de carga: las filas siguen ahi y
 * el fallo se ensena al final de la tabla.
 */
function statusOf(hasData: boolean, isError: boolean): GraphWindowState['status'] {
  if (hasData) return 'success';
  return isError ? 'error' : 'pending';
}
