import type { BridgeClient, GraphRow } from '@ramal/bridge';
import { useInfiniteQuery, useQueryClient } from '@tanstack/react-query';
import { useCallback, useEffect, useMemo } from 'react';

import {
  DEFAULT_PAGE_SIZE,
  FIRST_PAGE,
  graphWindowKey,
  isStaleWindow,
  mergePages,
  nextPageParam,
  shouldRetry,
  windowParams,
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
 *
 * Si el repositorio cambia entre dos paginas (un commit desde la terminal), el
 * motor responde `stale_window` y la consulta vuelve a empezar desde la fila 0.
 * Pegar la pagina nueva a las viejas mezclaria dos historiales: filas
 * repetidas o que faltan y carriles que saltan en el borde de la pagina.
 */
export function useGraphWindow(
  client: BridgeClient,
  repoId: string,
  options: UseGraphWindowOptions = {},
): GraphWindowState {
  const pageSize = options.pageSize ?? DEFAULT_PAGE_SIZE;
  const queryClient = useQueryClient();
  const queryKey = useMemo(() => graphWindowKey(repoId, pageSize), [repoId, pageSize]);
  const query = useInfiniteQuery({
    queryKey,
    queryFn: ({ pageParam, signal }) =>
      client.call('graph/window', windowParams(repoId, pageSize, pageParam), { signal }),
    initialPageParam: FIRST_PAGE,
    getNextPageParam: nextPageParam,
    retry: shouldRetry,
    staleTime: Number.POSITIVE_INFINITY,
    refetchOnWindowFocus: false,
    refetchOnReconnect: false,
  });
  const { data, error, fetchNextPage, refetch, hasNextPage, isFetchingNextPage } = query;
  const loaded = useMemo(() => mergePages(data?.pages ?? []), [data]);
  const stale = isStaleWindow(error);
  const loadMoreError = data !== undefined && query.isFetchNextPageError && !stale ? error : null;

  // Un efecto y no el callback de loadMore: tambien atrapa un refetch completo
  // (el host invalidando la consulta) que tropieza con el cambio a media lectura.
  useEffect(() => {
    if (stale) void queryClient.resetQueries({ queryKey, exact: true });
  }, [stale, queryClient, queryKey]);

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
