import { describe, expect, it } from 'vitest';

import { relativeTimeBucket } from './relative-time';

const NOW = Date.UTC(2026, 8, 30, 12, 0, 0);
const SECOND = 1000;
const MINUTE = 60 * SECOND;
const HOUR = 60 * MINUTE;
const DAY = 24 * HOUR;

function bucket(elapsed: number): ReturnType<typeof relativeTimeBucket> {
  return relativeTimeBucket(NOW - elapsed, NOW);
}

describe('relativeTimeBucket', () => {
  it('menos de un minuto es ahora, nunca "hace 0 min"', () => {
    expect(bucket(0)).toEqual({ unit: 'second', value: 0 });
    expect(bucket(59 * SECOND)).toEqual({ unit: 'second', value: 0 });
    expect(Object.is(bucket(0).value, -0)).toBe(false);
  });

  it('minutos hasta la hora', () => {
    expect(bucket(MINUTE)).toEqual({ unit: 'minute', value: -1 });
    expect(bucket(59 * MINUTE)).toEqual({ unit: 'minute', value: -59 });
  });

  it('1439 minutos son 23 horas y 24 horas ya son un dia', () => {
    expect(bucket(60 * MINUTE)).toEqual({ unit: 'hour', value: -1 });
    expect(bucket(1439 * MINUTE)).toEqual({ unit: 'hour', value: -23 });
    expect(bucket(24 * HOUR)).toEqual({ unit: 'day', value: -1 });
  });

  it('hay semanas, meses y anos', () => {
    expect(bucket(6 * DAY)).toEqual({ unit: 'day', value: -6 });
    expect(bucket(7 * DAY)).toEqual({ unit: 'week', value: -1 });
    expect(bucket(29 * DAY)).toEqual({ unit: 'week', value: -4 });
    expect(bucket(30 * DAY)).toEqual({ unit: 'month', value: -1 });
    expect(bucket(364 * DAY)).toEqual({ unit: 'month', value: -12 });
    expect(bucket(365 * DAY)).toEqual({ unit: 'year', value: -1 });
  });

  it('una fecha en el futuro cuenta como ahora', () => {
    expect(relativeTimeBucket(NOW + HOUR, NOW)).toEqual({ unit: 'second', value: 0 });
  });
});
