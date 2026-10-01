import type { HealthReport } from '../../domain/health/health-report';
import type { HealthGateway } from '../ports/health-gateway';

export type HealthProbe =
  | { readonly outcome: 'reachable'; readonly report: HealthReport }
  | { readonly outcome: 'unreachable' };

/**
 * Caso de uso "comprobar la salud de la API" de la ruta /status. Un fallo del
 * adaptador no es una excepcion para quien llama: es un resultado mas, el de API
 * inalcanzable. El motivo exacto queda en la consola del navegador, no en la
 * pantalla, porque no ayuda a quien la mira.
 */
export async function checkApiHealth(gateway: HealthGateway): Promise<HealthProbe> {
  try {
    const report = await gateway.readReadiness();
    return { outcome: 'reachable', report };
  } catch {
    return { outcome: 'unreachable' };
  }
}
