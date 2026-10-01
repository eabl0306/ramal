import type { ReleaseInfo } from '../../domain/downloads/release';

/**
 * De donde salen los datos de descarga. En el primer corte son fijos; cuando la
 * API sirva GET /v1/releases/latest (ADR 0016), cambia el adaptador y nada mas.
 */
export interface ReleaseCatalog {
  readonly latest: () => Promise<ReleaseInfo>;
}
