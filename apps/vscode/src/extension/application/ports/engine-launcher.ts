import type { EngineConnection } from './engine-connection';

/** Como termino el proceso del motor: codigo de salida, senal o fallo al lanzarlo. */
export interface EngineExit {
  readonly code: number | null;
  readonly signal: string | null;
  /** Fallo al arrancar el proceso (binario que no existe, permisos), si lo hubo. */
  readonly error?: string;
}

/** Un proceso del motor en marcha. */
export interface EngineProcess {
  readonly connection: EngineConnection;
  /** Se llama una sola vez, cuando el proceso termina por lo que sea. */
  onExit(listener: (exit: EngineExit) => void): void;
  /** Cierra su stdin para que termine solo y lo mata si no lo hace a tiempo. */
  stop(): void;
}

/** Lanza el proceso del motor. El supervisor decide cuando y cuantas veces. */
export interface EngineLauncher {
  launch(): EngineProcess;
}
