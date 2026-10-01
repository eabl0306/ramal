import {
  type EngineInfo,
  type ErrorKind,
  type JsonValue,
  PROTOCOL_VERSION,
  type RpcNotification,
  type RpcResponse,
} from '@ramal/bridge';

import type {
  EngineConnection,
  Unsubscribe,
} from '../extension/application/ports/engine-connection';
import type {
  EngineExit,
  EngineLauncher,
  EngineProcess,
} from '../extension/application/ports/engine-launcher';
import type { IdentifiedRpcRequest } from '../shared/messages';

type Responder = (request: IdentifiedRpcRequest) => Promise<RpcResponse>;

/** Respuesta correcta a una peticion. */
export function ok(request: IdentifiedRpcRequest, result: JsonValue): RpcResponse {
  return { jsonrpc: '2.0', id: request.id, result };
}

/** Respuesta con error del motor. */
export function fail(request: IdentifiedRpcRequest, kind: ErrorKind, code = -32_000): RpcResponse {
  return {
    jsonrpc: '2.0',
    id: request.id,
    error: { code, message: 'engine failed', data: { kind } },
  };
}

/** Lo que devuelve `engine/info`. */
export function engineInfo(protocolVersion: number = PROTOCOL_VERSION): EngineInfo {
  return {
    engineVersion: '0.1.0',
    protocolVersion,
    git: { path: '/usr/bin/git', version: '2.45.1', supported: true },
  };
}

/** Responde `engine/info` con la version dada y el resto con `{}`. */
export function infoResponder(protocolVersion: number = PROTOCOL_VERSION): Responder {
  return (request) =>
    Promise.resolve(
      request.method === 'engine/info' ? ok(request, engineInfo(protocolVersion)) : ok(request, {}),
    );
}

/** Conexion falsa con el motor: guarda lo que recibe y responde lo que se le diga. */
export class FakeEngine implements EngineConnection {
  readonly requests: IdentifiedRpcRequest[] = [];
  respond: Responder = infoResponder();
  readonly #listeners = new Set<(notification: RpcNotification) => void>();

  request(request: IdentifiedRpcRequest): Promise<RpcResponse> {
    this.requests.push(request);
    return this.respond(request);
  }

  onNotification(listener: (notification: RpcNotification) => void): Unsubscribe {
    this.#listeners.add(listener);
    return () => {
      this.#listeners.delete(listener);
    };
  }

  emit(notification: RpcNotification): void {
    for (const listener of this.#listeners) listener(notification);
  }

  get listenerCount(): number {
    return this.#listeners.size;
  }
}

/** Un proceso falso del motor que se puede "matar" desde el test. */
export class FakeProcess implements EngineProcess {
  readonly connection = new FakeEngine();
  stopped = false;
  readonly #listeners: ((exit: EngineExit) => void)[] = [];

  onExit(listener: (exit: EngineExit) => void): void {
    this.#listeners.push(listener);
  }

  stop(): void {
    this.stopped = true;
  }

  /** El proceso termina: las peticiones en vuelo fallarian en la conexion real. */
  exit(exit: EngineExit = { code: 1, signal: null }): void {
    for (const listener of this.#listeners) listener(exit);
  }
}

/** Lanzador falso: cada `launch` crea un proceso nuevo que el test puede manejar. */
export class FakeLauncher implements EngineLauncher {
  readonly processes: FakeProcess[] = [];
  /** Lo que respondera la conexion de cada proceso nuevo. */
  responder: Responder = infoResponder();
  failNext = false;

  launch(): EngineProcess {
    if (this.failNext) {
      this.failNext = false;
      throw new Error('spawn ENOENT');
    }
    const process = new FakeProcess();
    process.connection.respond = this.responder;
    this.processes.push(process);
    return process;
  }

  get last(): FakeProcess {
    const process = this.processes.at(-1);
    if (process === undefined) throw new Error('no process was launched');
    return process;
  }
}
