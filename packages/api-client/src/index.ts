// API publica de @git-rudder/api-client. Solo la importa apps/web/src/infrastructure
// (architecture.md 5.2): los adaptadores HTTP de la web hablan con la API a
// traves de este cliente y validan lo que reciben antes de pasarlo al dominio.
import type { components } from './generated/schema';

export { type ApiClient, type ApiClientOptions, createApiClient, normalizeBaseUrl } from './client';
export type { components, operations, paths } from './generated/schema';

/** Respuesta de /health/live y /health/ready. */
export type HealthResponse = components['schemas']['HealthResponse'];

/** Cuerpo de error del contrato: application/problem+json (RFC 9457). */
export type Problem = components['schemas']['Problem'];
