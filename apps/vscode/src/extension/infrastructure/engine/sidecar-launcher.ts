import type { Buffer } from 'node:buffer';
import { type ChildProcessWithoutNullStreams, spawn as nodeSpawn } from 'node:child_process';

import type {
  EngineExit,
  EngineLauncher,
  EngineProcess,
} from '../../application/ports/engine-launcher';
import type { Logger } from '../../application/ports/logger';

import { FrameDecoder } from './frame-codec';
import { JsonRpcClient } from './json-rpc-client';
import { LineSplitter } from './line-splitter';

/** Tiempo que se deja al sidecar para salir solo al cerrar su stdin. */
const STOP_GRACE_MS = 2000;

export type SpawnSidecar = (
  binary: string,
  args: readonly string[],
) => ChildProcessWithoutNullStreams;

export interface SidecarLauncherOptions {
  readonly binaryPath: string;
  readonly args: readonly string[];
  /** Log del host (canal "Ramal"). */
  readonly logger: Logger;
  /** Cada linea de stderr del sidecar: su log, que va al canal "Ramal Engine". */
  readonly onEngineLog: (line: string) => void;
  /** Para los tests; por defecto `child_process.spawn` con las tres tuberias. */
  readonly spawn?: SpawnSidecar;
}

const defaultSpawn: SpawnSidecar = (binary, args) =>
  nodeSpawn(binary, [...args], { stdio: ['pipe', 'pipe', 'pipe'], windowsHide: true });

/**
 * Lanza `ramal-vscode-engine` como proceso hijo. stdout es solo protocolo
 * (marcos `Content-Length`); stderr es su log. El sidecar termina cuando se
 * cierra su stdin, asi que parar es cerrar stdin y matar solo si no sale.
 */
export function createSidecarLauncher(options: SidecarLauncherOptions): EngineLauncher {
  return { launch: () => launchSidecar(options) };
}

function launchSidecar(options: SidecarLauncherOptions): EngineProcess {
  const child = (options.spawn ?? defaultSpawn)(options.binaryPath, options.args);
  const client = new JsonRpcClient((frame) => child.stdin.write(frame), options.logger);
  pipeStdout(child, client, options.logger);
  pipeStderr(child, options.onEngineLog);
  child.stdin.on('error', (error) => {
    options.logger.warn(`engine stdin closed: ${error.message}`);
  });
  const listeners: ((exit: EngineExit) => void)[] = [];
  let exit: EngineExit | undefined;
  const finish = (result: EngineExit): void => {
    if (exit !== undefined) return;
    exit = result;
    client.close(result.error ?? `exit code ${String(result.code)}`);
    for (const listener of listeners) listener(result);
  };
  child.on('error', (error) => {
    finish({ code: null, signal: null, error: `could not run the engine: ${error.message}` });
  });
  // `close` y no `exit`: llega cuando stdout ya se vacio, asi no se pierde la
  // ultima respuesta del proceso.
  child.on('close', (code, signal) => {
    finish({ code, signal });
  });
  return {
    connection: client,
    onExit: (listener) => {
      if (exit === undefined) listeners.push(listener);
      else listener(exit);
    },
    stop: () => {
      stopChild(child);
    },
  };
}

function pipeStdout(
  child: ChildProcessWithoutNullStreams,
  client: JsonRpcClient,
  logger: Logger,
): void {
  const decoder = new FrameDecoder();
  child.stdout.on('data', (chunk: Buffer) => {
    try {
      for (const body of decoder.push(chunk)) client.receive(body);
    } catch (error) {
      logger.error(`engine output is broken, stopping it: ${String(error)}`);
      child.kill();
    }
  });
}

function pipeStderr(child: ChildProcessWithoutNullStreams, onLine: (line: string) => void): void {
  const lines = new LineSplitter();
  child.stderr.on('data', (chunk: Buffer) => {
    for (const line of lines.push(chunk.toString('utf8'))) onLine(line);
  });
  child.stderr.on('end', () => {
    const rest = lines.flush();
    if (rest !== null) onLine(rest);
  });
}

function stopChild(child: ChildProcessWithoutNullStreams): void {
  if (child.exitCode !== null || child.signalCode !== null) return;
  child.stdin.end();
  const timer = setTimeout(() => {
    if (child.exitCode === null && child.signalCode === null) child.kill();
  }, STOP_GRACE_MS);
  timer.unref();
}
