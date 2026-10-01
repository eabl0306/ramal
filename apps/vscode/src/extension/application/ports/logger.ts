/**
 * Log del extension host. Lo implementa el canal de salida "Ramal"; los mensajes
 * van en ingles, como los del motor, porque son para diagnosticar.
 */
export interface Logger {
  debug(message: string): void;
  info(message: string): void;
  warn(message: string): void;
  error(message: string): void;
}
