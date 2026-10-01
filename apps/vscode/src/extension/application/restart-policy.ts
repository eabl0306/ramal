/** Esperas antes de cada reinicio seguido del motor (ADR 0010). */
export const RESTART_DELAYS_MS = [1000, 2000, 4000, 8000, 16_000] as const;

/** Ventana en la que se cuentan las caidas. */
export const CRASH_WINDOW_MS = 180_000;

/** Con estas caidas dentro de la ventana, el host deja de intentarlo. */
export const MAX_CRASHES_IN_WINDOW = 5;

export type RestartDecision =
  { readonly kind: 'restart'; readonly delayMs: number } | { readonly kind: 'give-up' };

/** Decide que hacer cuando el motor se cae. */
export interface RestartPolicy {
  /**
   * Apunta una caida a la hora `now` de un proceso que arranco en `startedAt` y
   * dice si reiniciar y tras cuanto.
   */
  recordCrash(now: number, startedAt: number): RestartDecision;
  /** Olvida las caidas: lo usa "Reintentar". */
  reset(): void;
}

/**
 * Dos cuentas distintas, como en el ADR: la espera crece con las caidas
 * seguidas (1, 2, 4, 8 y 16 segundos, y 16 a partir de ahi) y el abandono mira
 * cuantas caidas hubo en los ultimos tres minutos. Un proceso que aguanto la
 * ventana entera sin caerse vuelve a empezar por un segundo.
 */
export function createRestartPolicy(): RestartPolicy {
  let crashes: number[] = [];
  let consecutive = 0;
  return {
    recordCrash(now, startedAt) {
      crashes = [...crashes.filter((at) => now - at < CRASH_WINDOW_MS), now];
      if (crashes.length >= MAX_CRASHES_IN_WINDOW) return { kind: 'give-up' };
      consecutive = now - startedAt >= CRASH_WINDOW_MS ? 1 : consecutive + 1;
      const index = Math.min(consecutive, RESTART_DELAYS_MS.length) - 1;
      return { kind: 'restart', delayMs: RESTART_DELAYS_MS[index] ?? RESTART_DELAYS_MS[4] };
    },
    reset() {
      crashes = [];
      consecutive = 0;
    },
  };
}
