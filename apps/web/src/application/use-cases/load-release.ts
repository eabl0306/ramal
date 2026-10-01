import type { ReleaseInfo } from '../../domain/downloads/release';
import type { ReleaseCatalog } from '../ports/release-catalog';

export type ReleaseLoad =
  | { readonly outcome: 'loaded'; readonly release: ReleaseInfo }
  | { readonly outcome: 'unavailable' };

/**
 * Datos de descarga para la landing. Corre al prerenderizar: si no hay datos, la
 * pagina se construye igual sin la seccion de descargas (ADR 0015), y el build de
 * la web no depende de que la API este levantada.
 */
export async function loadRelease(catalog: ReleaseCatalog): Promise<ReleaseLoad> {
  try {
    return { outcome: 'loaded', release: await catalog.latest() };
  } catch {
    return { outcome: 'unavailable' };
  }
}
