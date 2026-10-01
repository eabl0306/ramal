/** Cancela un temporizador pendiente. */
export type CancelTimer = () => void;

/**
 * Reloj y temporizadores. Un puerto y no `setTimeout` a secas para que los
 * reintentos del supervisor se prueben con un reloj falso, sin esperar.
 */
export interface Timer {
  /** Milisegundos desde una referencia fija; solo se usan diferencias. */
  now(): number;
  setTimeout(callback: () => void, ms: number): CancelTimer;
}
