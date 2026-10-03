import type { HealthResponse } from '@git-rudder/api-client';
import { z } from 'zod';

import {
  type HealthReport,
  createHealthCheck,
  createHealthReport,
} from '../../domain/health/health-report';

/**
 * El contrato de /health/ready tal y como viaja por el cable. Los tipos de
 * openapi-typescript no validan nada en ejecucion: si la API cambia el JSON, el
 * fallo salta aqui y no a mitad de la interfaz. El `satisfies` ata el esquema al
 * tipo generado, asi que un cambio de openapi.json que no se refleje aqui rompe
 * el typecheck.
 */
const healthCheckDto = z.object({
  name: z.string().min(1),
  status: z.enum(['ok', 'error']),
  latencyMs: z.number().int().nonnegative(),
});

export const healthResponseSchema = z.object({
  status: z.enum(['ok', 'degraded']),
  checks: z.array(healthCheckDto),
  version: z.string(),
  checkedAt: z.iso.datetime({ offset: true }),
}) satisfies z.ZodType<HealthResponse>;

export function toHealthReport(dto: HealthResponse): HealthReport {
  return createHealthReport({
    checks: dto.checks.map((check) => createHealthCheck(check.name, check.status, check.latencyMs)),
    version: dto.version,
    checkedAt: new Date(dto.checkedAt),
    reportedStatus: dto.status,
  });
}
