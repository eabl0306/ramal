import type { RpcNotification, RpcResponse } from '@ramal/bridge';

import type { IdentifiedRpcRequest } from '../../../shared/messages';
import { EngineUnavailableError, describeError } from '../engine-errors';
import type { EngineConnection, Unsubscribe } from '../ports/engine-connection';
import type { EngineExit, EngineLauncher, EngineProcess } from '../ports/engine-launcher';
import type { Logger } from '../ports/logger';
import type { CancelTimer, Timer } from '../ports/timer';
import { type RestartPolicy, createRestartPolicy } from '../restart-policy';

import type { ProtocolCheck } from './check-engine-protocol';

type Mismatch = Extract<ProtocolCheck, { outcome: 'mismatch' }>;

export interface SuperviseEngineDeps {
  readonly launcher: EngineLauncher;
  readonly timer: Timer;
  readonly logger: Logger;
  /** Comprueba un proceso recien arrancado: `engine/info` y la version del protocolo. */
  readonly handshake: (connection: EngineConnection) => Promise<ProtocolCheck>;
  /** Se llama al dejar de reintentar; `retry` vuelve a empezar desde cero. */
  readonly onGiveUp: (retry: () => void) => void;
  /** Se llama si el motor habla otra version del protocolo. No se reintenta. */
  readonly onProtocolMismatch: (mismatch: Mismatch) => void;
  readonly policy?: RestartPolicy;
}

export type EngineStateKind = 'idle' | 'starting' | 'running' | 'waiting' | 'failed' | 'stopped';

/** El motor visto desde el resto del host: una conexion que se repara sola. */
export interface SupervisedEngine extends EngineConnection {
  readonly state: EngineStateKind;
  /** Vuelve a arrancar despues de que el host dejara de intentarlo. */
  retry(): void;
  /** Para el motor para siempre (al desactivar la extension). */
  stop(): void;
}

type State =
  | { readonly kind: 'idle' | 'failed' | 'stopped' }
  | { readonly kind: 'waiting'; readonly cancel: CancelTimer }
  | {
      readonly kind: 'starting';
      readonly process: EngineProcess;
      readonly ready: Promise<EngineConnection>;
    }
  | { readonly kind: 'running'; readonly process: EngineProcess };

/**
 * Arranca el sidecar la primera vez que hace falta, comprueba su protocolo y lo
 * reinicia si se cae, con la espera creciente de la politica de reinicio. Las
 * peticiones que llegan mientras espera o despues de rendirse fallan al momento
 * con `EngineUnavailableError`; las que estaban en vuelo las rechaza la conexion
 * del proceso al cerrarse.
 */
class EngineSupervisor implements SupervisedEngine {
  readonly #deps: SuperviseEngineDeps;
  readonly #policy: RestartPolicy;
  readonly #listeners = new Set<(notification: RpcNotification) => void>();
  #state: State = { kind: 'idle' };
  #startedAt = 0;

  constructor(deps: SuperviseEngineDeps) {
    this.#deps = deps;
    this.#policy = deps.policy ?? createRestartPolicy();
  }

  get state(): EngineStateKind {
    return this.#state.kind;
  }

  async request(request: IdentifiedRpcRequest): Promise<RpcResponse> {
    const connection = await this.#connection();
    return connection.request(request);
  }

  onNotification(listener: (notification: RpcNotification) => void): Unsubscribe {
    this.#listeners.add(listener);
    return () => {
      this.#listeners.delete(listener);
    };
  }

  retry(): void {
    if (this.#state.kind !== 'failed') return;
    this.#policy.reset();
    this.#state = { kind: 'idle' };
    this.#start().catch(() => undefined);
  }

  stop(): void {
    const previous = this.#state;
    this.#state = { kind: 'stopped' };
    if (previous.kind === 'waiting') previous.cancel();
    if (previous.kind === 'starting' || previous.kind === 'running') previous.process.stop();
  }

  #connection(): Promise<EngineConnection> {
    const state = this.#state;
    switch (state.kind) {
      case 'running':
        return Promise.resolve(state.process.connection);
      case 'starting':
        return state.ready;
      case 'idle':
        return this.#start();
      case 'waiting':
      case 'failed':
      case 'stopped':
        return Promise.reject(new EngineUnavailableError(`the engine is ${state.kind}`));
    }
  }

  #start(): Promise<EngineConnection> {
    this.#startedAt = this.#deps.timer.now();
    let process: EngineProcess;
    try {
      process = this.#deps.launcher.launch();
    } catch (error) {
      this.#deps.logger.error(`could not launch the engine: ${describeError(error)}`);
      this.#handleCrash();
      return Promise.reject(new EngineUnavailableError('could not launch the engine'));
    }
    const ready = this.#handshake(process);
    this.#state = { kind: 'starting', process, ready };
    process.onExit((exit) => {
      this.#onExit(process, exit);
    });
    process.connection.onNotification((notification) => {
      for (const listener of this.#listeners) listener(notification);
    });
    return ready;
  }

  async #handshake(process: EngineProcess): Promise<EngineConnection> {
    const check = await this.#deps.handshake(process.connection);
    if (!this.#isCurrent(process)) {
      throw new EngineUnavailableError('the engine stopped while starting');
    }
    if (check.outcome === 'compatible') {
      this.#state = { kind: 'running', process };
      const { engineVersion, git } = check.info;
      this.#deps.logger.info(`engine ${engineVersion} ready with git ${git.version}`);
      return process.connection;
    }
    if (check.outcome === 'mismatch') {
      this.#state = { kind: 'failed' };
      process.stop();
      this.#deps.logger.error(
        `engine speaks protocol ${String(check.engineVersion)}, expected ${String(check.expected)}`,
      );
      this.#deps.onProtocolMismatch(check);
      throw new EngineUnavailableError('the engine speaks another protocol version');
    }
    // Si engine/info fallo y el proceso sigue vivo, se para: su salida cuenta
    // como una caida y pasa por la politica de reinicio.
    this.#deps.logger.error(`engine handshake failed: ${describeError(check.error)}`);
    process.stop();
    throw new EngineUnavailableError('the engine did not answer engine/info');
  }

  #onExit(process: EngineProcess, exit: EngineExit): void {
    if (!this.#isCurrent(process)) return;
    this.#deps.logger.warn(`engine exited: ${describeExit(exit)}`);
    this.#handleCrash();
  }

  #handleCrash(): void {
    const decision = this.#policy.recordCrash(this.#deps.timer.now(), this.#startedAt);
    if (decision.kind === 'give-up') {
      this.#state = { kind: 'failed' };
      this.#deps.logger.error('engine crashed too many times; not restarting it');
      this.#deps.onGiveUp(() => {
        this.retry();
      });
      return;
    }
    const cancel = this.#deps.timer.setTimeout(() => {
      this.#restart();
    }, decision.delayMs);
    this.#state = { kind: 'waiting', cancel };
    this.#deps.logger.info(`restarting the engine in ${String(decision.delayMs)} ms`);
  }

  #restart(): void {
    if (this.#state.kind !== 'waiting') return;
    this.#start().catch(() => undefined);
  }

  #isCurrent(process: EngineProcess): boolean {
    const state = this.#state;
    return (state.kind === 'starting' || state.kind === 'running') && state.process === process;
  }
}

function describeExit(exit: EngineExit): string {
  if (exit.error !== undefined) return exit.error;
  if (exit.signal !== null) return `signal ${exit.signal}`;
  return `code ${String(exit.code)}`;
}

/** Crea el supervisor. No arranca nada hasta la primera peticion. */
export function superviseEngine(deps: SuperviseEngineDeps): SupervisedEngine {
  return new EngineSupervisor(deps);
}
