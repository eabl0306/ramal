import { describe, expect, it } from 'vitest';

import { createHealthCheck, createHealthReport } from '../../domain/health/health-report';
import { type HealthGateway, HealthGatewayError } from '../ports/health-gateway';

import { checkApiHealth } from './check-api-health';

const REPORT = createHealthReport({
  checks: [createHealthCheck('database', 'error', 12)],
  version: '0.1.0',
  checkedAt: new Date('2026-09-30T10:00:00Z'),
});

describe('checkApiHealth', () => {
  it('devuelve el informe cuando la API responde, aunque venga degradado', async () => {
    const gateway: HealthGateway = { readReadiness: () => Promise.resolve(REPORT) };

    await expect(checkApiHealth(gateway)).resolves.toEqual({
      outcome: 'reachable',
      report: REPORT,
    });
  });

  it('convierte un fallo del adaptador en API inalcanzable, sin lanzar', async () => {
    const gateway: HealthGateway = {
      readReadiness: () => Promise.reject(new HealthGatewayError('sin red')),
    };

    await expect(checkApiHealth(gateway)).resolves.toEqual({ outcome: 'unreachable' });
  });

  it('nombra su error para distinguirlo en la consola', () => {
    expect(new HealthGatewayError('x').name).toBe('HealthGatewayError');
  });
});
