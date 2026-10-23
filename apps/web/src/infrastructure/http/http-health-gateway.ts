import { type ApiClient, createApiClient } from '@git-rudder/api-client';

import { type HealthGateway, HealthGatewayError } from '../../application/ports/health-gateway';

import { healthResponseSchema, toHealthReport } from './health-dto';

export interface HttpHealthGatewayOptions {
  /** Vacio es el mismo origen, el caso normal detras del gateway. */
  readonly baseUrl: string;
  readonly timeoutMs?: number;
  readonly fetch?: (input: Request) => Promise<Response>;
}

const DEFAULT_TIMEOUT_MS = 5000;

/** 200 es operativo y 503 degradado: los dos traen el informe en el cuerpo. */
const READABLE_STATUSES = new Set([200, 503]);

// Sin tipo de retorno escrito a proposito: el que infiere openapi-fetch para esta
// ruta concreta es mas preciso que cualquiera que se pueda nombrar a mano.
async function requestReadiness(client: ApiClient, timeoutMs: number) {
  try {
    return await client.GET('/health/ready', { signal: AbortSignal.timeout(timeoutMs) });
  } catch (error: unknown) {
    // Red caida, timeout o un 200 que no es JSON (una ruta de la API que el
    // gateway mando a la web contesta HTML): para la web es lo mismo.
    throw new HealthGatewayError('No se pudo leer el estado de la API', { cause: error });
  }
}

/**
 * Adaptador de salida sobre HTTP con @git-rudder/api-client. Un 503 no es un error de
 * transporte: la API usa ese codigo para decir "estoy degradada", asi que su
 * cuerpo tambien se lee.
 */
export function createHttpHealthGateway(options: HttpHealthGatewayOptions): HealthGateway {
  const client = createApiClient({
    baseUrl: options.baseUrl,
    ...(options.fetch === undefined ? {} : { fetch: options.fetch }),
  });
  return {
    readReadiness: async () => {
      const result = await requestReadiness(client, options.timeoutMs ?? DEFAULT_TIMEOUT_MS);
      const status = result.response.status;
      if (!READABLE_STATUSES.has(status)) {
        throw new HealthGatewayError(`La API respondio con estado ${String(status)}`);
      }
      const parsed = healthResponseSchema.safeParse(result.data ?? result.error);
      if (!parsed.success) {
        throw new HealthGatewayError('La respuesta de salud no cumple el contrato', {
          cause: parsed.error,
        });
      }
      return toHealthReport(parsed.data);
    },
  };
}
