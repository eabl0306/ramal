import type { HealthGateway } from '../application/ports/health-gateway';
import type { ReleaseCatalog } from '../application/ports/release-catalog';
import { createUnavailableAuthGateway } from '../infrastructure/auth/unavailable-auth-gateway';
import { createBrowserClipboard } from '../infrastructure/browser/browser-clipboard';
import { createNavigatorEnvironment } from '../infrastructure/browser/navigator-environment';
import { createHttpHealthGateway } from '../infrastructure/http/http-health-gateway';
import { createStaticReleaseCatalog } from '../infrastructure/releases/static-release-catalog';
import { createLocalPreferencesStore } from '../infrastructure/storage/local-preferences-store';
import type { UiPorts } from '../ui/ports/ui-ports';

/** Lo unico que la web necesita saber del entorno. */
export interface WebEnv {
  readonly VITE_API_URL?: string;
}

/** Todo lo que la interfaz y las rutas reciben ya montado. */
export interface Container extends UiPorts {
  readonly health: HealthGateway;
  readonly releases: ReleaseCatalog;
}

/**
 * Vacio es el mismo origen, que es como se sirve: el gateway pone la API bajo el
 * dominio de la web y la API no manda CORS (ADR 0007). VITE_API_URL solo hace
 * falta para apuntar a una API de otro sitio.
 */
export function resolveApiUrl(env: WebEnv): string {
  return env.VITE_API_URL?.trim() ?? '';
}

const HEALTH_TIMEOUT_MS = 5000;

/**
 * Raiz de composicion: el unico sitio donde la interfaz y los adaptadores se
 * conocen. Cambiar de adaptador (el AuthGateway de verdad, los datos de descarga
 * de la API) se hace aqui y en ningun otro lado. Ningun adaptador toca el
 * navegador al construirse, porque el contenedor tambien existe al prerenderizar.
 */
export function createContainer(env: WebEnv): Container {
  return {
    health: createHttpHealthGateway({ baseUrl: resolveApiUrl(env), timeoutMs: HEALTH_TIMEOUT_MS }),
    releases: createStaticReleaseCatalog(),
    auth: createUnavailableAuthGateway(),
    preferences: createLocalPreferencesStore(),
    visitor: createNavigatorEnvironment(),
    clipboard: createBrowserClipboard(),
  };
}
