import type { HealthReport } from '../../domain/health/health-report';

/**
 * Puerto de salida: el nucleo necesita saber si la API puede atender, sin decir
 * como se averigua. El adaptador HTTP vive en infrastructure.
 */
export interface HealthGateway {
  /** Lee /health/ready. Un 503 no es un fallo: es un informe degradado. */
  readonly readReadiness: () => Promise<HealthReport>;
}

/** Fallo de un adaptador al hablar con el exterior: red, timeout o un cuerpo que no cumple. */
export class HealthGatewayError extends Error {
  constructor(message: string, options?: ErrorOptions) {
    super(message, options);
    this.name = 'HealthGatewayError';
  }
}
