/**
 * Tramos del tiempo relativo ("hace 5 minutos", "hace 2 semanas"). Aqui se
 * decide la unidad y el numero; el texto lo pone formatRelativeTime de
 * @git-rudder/i18n con Intl.RelativeTimeFormat, que ya sabe decir "ayer" o "ahora".
 *
 * Corrige el `ago()` del prototipo (gaps B-48): nada de "hace 0 min", 1439
 * minutos son 23 horas y no "24 h", y hay semanas, meses y anos. Se redondea
 * hacia abajo para no anunciar una unidad que todavia no se ha cumplido.
 */

export type RelativeTimeUnit = 'second' | 'minute' | 'hour' | 'day' | 'week' | 'month' | 'year';

export interface RelativeTimeBucket {
  readonly unit: RelativeTimeUnit;
  /** Negativo en el pasado, como lo espera Intl.RelativeTimeFormat. 0 es "ahora". */
  readonly value: number;
}

const SECOND = 1000;
const MINUTE = 60 * SECOND;
const HOUR = 60 * MINUTE;
const DAY = 24 * HOUR;
const WEEK = 7 * DAY;
const MONTH = 30 * DAY;
const YEAR = 365 * DAY;

/** Desde que duracion se usa cada unidad. Se recorre de arriba abajo. */
const STEPS: readonly (readonly [RelativeTimeUnit, number, number])[] = [
  ['year', YEAR, YEAR],
  ['month', MONTH, MONTH],
  ['week', WEEK, WEEK],
  ['day', DAY, DAY],
  ['hour', HOUR, HOUR],
  ['minute', MINUTE, MINUTE],
];

function past(value: number): number {
  return value === 0 ? 0 : -value;
}

/**
 * `then` y `now` son milisegundos desde la epoca. El reloj no se lee aqui: lo
 * pasa quien llama, asi que la funcion es pura y se prueba en los bordes.
 * Una fecha en el futuro (relojes desincronizados) se trata como "ahora".
 */
export function relativeTimeBucket(then: number, now: number): RelativeTimeBucket {
  const elapsed = Math.max(0, now - then);
  for (const [unit, from, size] of STEPS) {
    if (elapsed >= from) return { unit, value: past(Math.floor(elapsed / size)) };
  }
  return { unit: 'second', value: 0 };
}
