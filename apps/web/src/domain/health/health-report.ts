/**
 * Informe de salud de la API tal y como lo entiende la web. Modelo puro: aqui no
 * hay HTTP, ni JSON, ni React. Lo construye el adaptador HTTP a partir de lo que
 * responde /health/ready.
 */
export type CheckStatus = 'ok' | 'error';

export interface HealthCheck {
  /** Nombre estable del componente (`database`); el texto visible lo pone la interfaz. */
  readonly name: string;
  readonly status: CheckStatus;
  readonly latencyMs: number;
}

export type HealthStatus = 'ok' | 'degraded';

export interface HealthReport {
  readonly status: HealthStatus;
  readonly checks: readonly HealthCheck[];
  readonly version: string;
  readonly checkedAt: Date;
}

export interface HealthReportInput {
  readonly checks: readonly HealthCheck[];
  readonly version: string;
  readonly checkedAt: Date;
  readonly reportedStatus?: HealthStatus;
}

export function isFailing(check: HealthCheck): boolean {
  return check.status === 'error';
}

/** Una latencia negativa o con decimales no significa nada en la pantalla. */
export function createHealthCheck(
  name: string,
  status: CheckStatus,
  latencyMs: number,
): HealthCheck {
  return { name, status, latencyMs: Math.max(0, Math.round(latencyMs)) };
}

/**
 * Regla de degradacion: el servicio esta degradado si falla al menos una
 * comprobacion, o si el propio servicio se declara degradado aunque sus
 * comprobaciones salgan bien. Nunca al reves: un "ok" del servidor no borra una
 * comprobacion en error.
 */
export function deriveStatus(
  checks: readonly HealthCheck[],
  reportedStatus?: HealthStatus,
): HealthStatus {
  if (checks.some(isFailing)) return 'degraded';
  return reportedStatus === 'degraded' ? 'degraded' : 'ok';
}

export function createHealthReport(input: HealthReportInput): HealthReport {
  return {
    status: deriveStatus(input.checks, input.reportedStatus),
    checks: [...input.checks],
    version: input.version.trim(),
    checkedAt: input.checkedAt,
  };
}
