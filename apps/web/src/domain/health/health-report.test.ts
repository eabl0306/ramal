import { describe, expect, it } from 'vitest';

import { createHealthCheck, createHealthReport, deriveStatus, isFailing } from './health-report';

const CHECKED_AT = new Date('2026-09-30T10:00:00Z');

describe('createHealthCheck', () => {
  it('redondea la latencia y nunca la deja negativa', () => {
    expect(createHealthCheck('database', 'ok', 3.6).latencyMs).toBe(4);
    expect(createHealthCheck('database', 'ok', -2).latencyMs).toBe(0);
  });

  it('marca como fallida solo la comprobacion en error', () => {
    expect(isFailing(createHealthCheck('database', 'error', 1))).toBe(true);
    expect(isFailing(createHealthCheck('database', 'ok', 1))).toBe(false);
  });
});

describe('deriveStatus', () => {
  const ok = createHealthCheck('database', 'ok', 2);
  const broken = createHealthCheck('database', 'error', 2);

  it('esta operativo si todo responde y el servidor no dice lo contrario', () => {
    expect(deriveStatus([ok])).toBe('ok');
    expect(deriveStatus([], 'ok')).toBe('ok');
  });

  it('esta degradado si falla una comprobacion aunque el servidor diga ok', () => {
    expect(deriveStatus([ok, broken], 'ok')).toBe('degraded');
  });

  it('esta degradado si el servidor lo declara aunque las comprobaciones salgan bien', () => {
    expect(deriveStatus([ok], 'degraded')).toBe('degraded');
  });
});

describe('createHealthReport', () => {
  it('copia las comprobaciones, recorta la version y conserva el instante', () => {
    const checks = [createHealthCheck('database', 'ok', 3)];
    const report = createHealthReport({ checks, version: ' 0.1.0 ', checkedAt: CHECKED_AT });

    expect(report).toEqual({ status: 'ok', checks, version: '0.1.0', checkedAt: CHECKED_AT });
    expect(report.checks).not.toBe(checks);
  });
});
