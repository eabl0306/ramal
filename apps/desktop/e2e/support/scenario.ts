import type { GraphRow, RepoHandle, RpcError } from '@ramal/bridge';

/**
 * Lo que contesta el host simulado en una prueba: que pasa al pedir un
 * repositorio y que historial devuelve `graph/window`. Viaja a la pagina como
 * JSON antes de que cargue la app.
 */
export interface E2eScenario {
  readonly pick:
    | { readonly outcome: 'opened'; readonly repo: RepoHandle }
    | { readonly outcome: 'cancelled' }
    | { readonly outcome: 'failed'; readonly error: RpcError };
  readonly rows: readonly GraphRow[];
}

/** Una llamada que el webview hizo al host: el comando y, si es `rpc`, el metodo. */
export interface E2eCall {
  readonly command: string;
  readonly method?: string;
}

declare global {
  interface Window {
    __RAMAL_E2E__?: E2eScenario;
    __RAMAL_E2E_CALLS__?: E2eCall[];
  }
}
