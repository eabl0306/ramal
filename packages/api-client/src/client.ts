import createClient, { type Client } from 'openapi-fetch';

import type { paths } from './generated/schema';

/** Cliente tipado de la Ramal API: cada ruta y cada respuesta salen de openapi.json. */
export type ApiClient = Client<paths>;

export interface ApiClientOptions {
  /**
   * Origen de la API. Vacio es el mismo origen, que es como se sirve: el gateway
   * pone la web y la API bajo un solo dominio y la API no manda CORS
   * (architecture.md 6.3). Solo hace falta para apuntar a una API de otro sitio.
   */
  readonly baseUrl?: string;
  /** fetch propio, para los tests de los adaptadores. */
  readonly fetch?: (input: Request) => Promise<Response>;
}

/** Sin barra final: openapi-fetch concatena el origen y una ruta que ya empieza por "/". */
export function normalizeBaseUrl(baseUrl: string): string {
  return baseUrl.trim().replace(/\/+$/u, '');
}

/**
 * El unico sitio que construye el cliente. Pide JSON siempre: la API contesta
 * problem+json en los errores, y es JSON igual.
 */
export function createApiClient(options: ApiClientOptions = {}): ApiClient {
  return createClient<paths>({
    baseUrl: normalizeBaseUrl(options.baseUrl ?? ''),
    headers: { accept: 'application/json' },
    ...(options.fetch === undefined ? {} : { fetch: options.fetch }),
  });
}
