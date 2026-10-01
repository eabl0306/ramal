import { Buffer } from 'node:buffer';
import type { ChildProcessWithoutNullStreams } from 'node:child_process';
import { EventEmitter } from 'node:events';
import { PassThrough } from 'node:stream';

import { afterEach, describe, expect, it, vi } from 'vitest';

import { RecordingLogger, flush } from '../../../test/host-fakes';
import { EngineUnavailableError } from '../../application/engine-errors';
import type { EngineExit } from '../../application/ports/engine-launcher';

import { FrameDecoder, encodeFrame } from './frame-codec';
import { createSidecarLauncher } from './sidecar-launcher';

/** Un proceso hijo de mentira con sus tres tuberias. */
class FakeChild extends EventEmitter {
  readonly stdin = new PassThrough();
  readonly stdout = new PassThrough();
  readonly stderr = new PassThrough();
  exitCode: number | null = null;
  signalCode: string | null = null;
  killed = false;

  kill(): boolean {
    this.killed = true;
    this.signalCode = 'SIGTERM';
    this.emit('close', null, 'SIGTERM');
    return true;
  }
}

const REQUEST = { jsonrpc: '2.0', id: 1, method: 'engine/info', params: {} } as const;

function launch(): {
  child: FakeChild;
  process: ReturnType<ReturnType<typeof createSidecarLauncher>['launch']>;
  logger: RecordingLogger;
  engineLog: string[];
  spawned: { binary: string; args: readonly string[] }[];
} {
  const child = new FakeChild();
  const logger = new RecordingLogger();
  const engineLog: string[] = [];
  const spawned: { binary: string; args: readonly string[] }[] = [];
  const launcher = createSidecarLauncher({
    binaryPath: '/ext/dist/bin/ramal-vscode-engine',
    args: ['--log-level', 'debug'],
    logger,
    onEngineLog: (line) => engineLog.push(line),
    spawn: (binary, args) => {
      spawned.push({ binary, args });
      return child as unknown as ChildProcessWithoutNullStreams;
    },
  });
  return { child, process: launcher.launch(), logger, engineLog, spawned };
}

afterEach(() => {
  vi.useRealTimers();
});

describe('proceso del sidecar', () => {
  it('lanza el binario con sus argumentos y habla en marcos por stdin y stdout', async () => {
    const { child, process, spawned } = launch();
    expect(spawned).toEqual([
      { binary: '/ext/dist/bin/ramal-vscode-engine', args: ['--log-level', 'debug'] },
    ]);
    const written: string[] = [];
    const decoder = new FrameDecoder();
    child.stdin.on('data', (chunk: Buffer) => written.push(...decoder.push(chunk)));
    const pending = process.connection.request(REQUEST);
    await flush();
    expect(written.map((body) => JSON.parse(body) as unknown)).toEqual([REQUEST]);
    child.stdout.write(encodeFrame('{"jsonrpc":"2.0","id":1,"result":{"ok":true}}'));
    await expect(pending).resolves.toEqual({ jsonrpc: '2.0', id: 1, result: { ok: true } });
  });

  it('vuelca su stderr linea a linea en el log del motor', async () => {
    const { child, engineLog } = launch();
    child.stderr.write('INFO engine ready\nWARN half');
    child.stderr.end(' line');
    await new Promise((resolve) => child.stderr.on('end', resolve));
    expect(engineLog).toEqual(['INFO engine ready', 'WARN half line']);
  });

  it('al terminar avisa una vez y las peticiones pendientes fallan', async () => {
    const { child, process } = launch();
    const exits: EngineExit[] = [];
    process.onExit((exit) => exits.push(exit));
    const pending = process.connection.request(REQUEST);
    child.emit('close', 101, null);
    child.emit('close', 101, null);
    await expect(pending).rejects.toBeInstanceOf(EngineUnavailableError);
    expect(exits).toEqual([{ code: 101, signal: null }]);
    const late: EngineExit[] = [];
    process.onExit((exit) => late.push(exit));
    expect(late).toEqual([{ code: 101, signal: null }]);
  });

  it('si no se puede lanzar, termina con el motivo', () => {
    const { child, process } = launch();
    const exits: EngineExit[] = [];
    process.onExit((exit) => exits.push(exit));
    child.emit('error', new Error('spawn ENOENT'));
    expect(exits).toEqual([
      { code: null, signal: null, error: 'could not run the engine: spawn ENOENT' },
    ]);
  });

  it('parar cierra stdin y lo mata si no sale a tiempo', () => {
    vi.useFakeTimers();
    const { child, process } = launch();
    process.stop();
    expect(child.stdin.writableEnded).toBe(true);
    expect(child.killed).toBe(false);
    vi.advanceTimersByTime(2000);
    expect(child.killed).toBe(true);
    process.stop();
  });

  it('parar un proceso que ya salio no hace nada', () => {
    vi.useFakeTimers();
    const { child, process } = launch();
    child.exitCode = 0;
    process.stop();
    vi.advanceTimersByTime(5000);
    expect(child.killed).toBe(false);
  });

  it('una salida rota mata el proceso en vez de seguir leyendo basura', () => {
    const { child, logger } = launch();
    child.stdout.write(Buffer.from('Content-Length: nope\r\n\r\n'));
    expect(child.killed).toBe(true);
    expect(logger.messages('error')[0]).toContain('engine output is broken');
  });

  it('registra un stdin que se cierra con error', () => {
    const { child, logger } = launch();
    child.stdin.emit('error', new Error('EPIPE'));
    expect(logger.messages('warn')).toEqual(['engine stdin closed: EPIPE']);
  });
});
