import { afterEach, describe, expect, it, vi } from 'vitest';

import { systemTimer } from './system-timer';

afterEach(() => {
  vi.useRealTimers();
});

describe('reloj del sistema', () => {
  it('ejecuta lo programado y se puede cancelar', () => {
    vi.useFakeTimers();
    const fired: string[] = [];
    systemTimer.setTimeout(() => fired.push('a'), 1000);
    const cancel = systemTimer.setTimeout(() => fired.push('b'), 1000);
    cancel();
    vi.advanceTimersByTime(1000);
    expect(fired).toEqual(['a']);
  });

  it('mide el tiempo hacia delante', () => {
    const first = systemTimer.now();
    expect(systemTimer.now()).toBeGreaterThanOrEqual(first);
  });
});
