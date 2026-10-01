import { describe, expect, it } from 'vitest';

import { CRASH_WINDOW_MS, createRestartPolicy } from './restart-policy';

const SECOND = 1000;

describe('politica de reinicio del motor', () => {
  it('espera 1, 2, 4 y 8 segundos y se rinde con la quinta caida en tres minutos', () => {
    const policy = createRestartPolicy();
    let now = 0;
    const delays: unknown[] = [];
    for (let crash = 0; crash < 5; crash += 1) {
      now += SECOND;
      const decision = policy.recordCrash(now, now - 100);
      delays.push(decision.kind === 'restart' ? decision.delayMs : decision.kind);
      if (decision.kind === 'restart') now += decision.delayMs;
    }
    expect(delays).toEqual([1000, 2000, 4000, 8000, 'give-up']);
  });

  it('llega a 16 segundos y se queda ahi si las caidas estan espaciadas', () => {
    const policy = createRestartPolicy();
    const delays: number[] = [];
    for (let crash = 0; crash < 7; crash += 1) {
      const now = crash * 50 * SECOND;
      const decision = policy.recordCrash(now, now - 10 * SECOND);
      if (decision.kind === 'restart') delays.push(decision.delayMs);
    }
    expect(delays).toEqual([1000, 2000, 4000, 8000, 16_000, 16_000, 16_000]);
  });

  it('un proceso que aguanto la ventana entera vuelve a empezar por un segundo', () => {
    const policy = createRestartPolicy();
    policy.recordCrash(0, 0);
    policy.recordCrash(10 * SECOND, 5 * SECOND);
    const later = 10 * SECOND + CRASH_WINDOW_MS;
    expect(policy.recordCrash(later, 10 * SECOND)).toEqual({ kind: 'restart', delayMs: 1000 });
  });

  it('reintentar olvida las caidas anteriores', () => {
    const policy = createRestartPolicy();
    for (let crash = 0; crash < 5; crash += 1) policy.recordCrash(crash, crash);
    policy.reset();
    expect(policy.recordCrash(10, 10)).toEqual({ kind: 'restart', delayMs: 1000 });
  });
});
