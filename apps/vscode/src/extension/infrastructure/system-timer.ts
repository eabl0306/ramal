import type { Timer } from '../application/ports/timer';

/**
 * El reloj y los temporizadores de Node. `performance.now` y no `Date.now`: el
 * supervisor solo mide intervalos, y un cambio de hora del sistema no debe
 * contar como tres minutos sin caidas.
 */
export const systemTimer: Timer = {
  now: () => performance.now(),
  setTimeout: (callback, ms) => {
    const handle = setTimeout(callback, ms);
    // Un reinicio pendiente no debe mantener vivo el extension host al cerrar.
    handle.unref();
    return () => {
      clearTimeout(handle);
    };
  },
};
