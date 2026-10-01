import type { RpcNotification } from '@ramal/bridge';
import { describe, expect, it } from 'vitest';

import { FakeLauncher, infoResponder, ok } from '../../../test/engine-fakes';
import { FakeTimer, RecordingLogger, flush } from '../../../test/host-fakes';
import { EngineUnavailableError } from '../engine-errors';
import { createRequestIds } from '../request-ids';

import { checkEngineProtocol } from './check-engine-protocol';
import { type SupervisedEngine, superviseEngine } from './supervise-engine';

const WINDOW_REQUEST = {
  jsonrpc: '2.0',
  id: 900,
  method: 'graph/window',
  params: { repoId: 'repo-1', offset: 0, limit: 10 },
} as const;

interface Harness {
  readonly engine: SupervisedEngine;
  readonly launcher: FakeLauncher;
  readonly timer: FakeTimer;
  readonly logger: RecordingLogger;
  readonly giveUps: (() => void)[];
  readonly mismatches: unknown[];
}

function harness(): Harness {
  const launcher = new FakeLauncher();
  const timer = new FakeTimer();
  const logger = new RecordingLogger();
  const giveUps: (() => void)[] = [];
  const mismatches: unknown[] = [];
  const ids = createRequestIds();
  const engine = superviseEngine({
    launcher,
    timer,
    logger,
    handshake: (connection) => checkEngineProtocol(connection, ids, 1),
    onGiveUp: (retry) => giveUps.push(retry),
    onProtocolMismatch: (mismatch) => mismatches.push(mismatch),
  });
  return { engine, launcher, timer, logger, giveUps, mismatches };
}

/** Hace caer el ultimo proceso y deja correr lo pendiente. */
async function crash(h: Harness): Promise<void> {
  h.launcher.last.exit({ code: 101, signal: null });
  await flush();
}

describe('supervision del motor', () => {
  it('no arranca nada hasta la primera peticion', () => {
    const h = harness();
    expect(h.launcher.processes).toHaveLength(0);
    expect(h.engine.state).toBe('idle');
  });

  it('arranca con la primera peticion, comprueba engine/info y despues la reenvia', async () => {
    const h = harness();
    const response = await h.engine.request(WINDOW_REQUEST);
    expect(response).toEqual(ok(WINDOW_REQUEST, {}));
    expect(h.launcher.processes).toHaveLength(1);
    expect(h.launcher.last.connection.requests.map((request) => request.method)).toEqual([
      'engine/info',
      'graph/window',
    ]);
    expect(h.engine.state).toBe('running');
  });

  it('dos peticiones a la vez comparten el mismo arranque', async () => {
    const h = harness();
    await Promise.all([
      h.engine.request(WINDOW_REQUEST),
      h.engine.request({ ...WINDOW_REQUEST, id: 901 }),
    ]);
    expect(h.launcher.processes).toHaveLength(1);
  });

  it('si se cae, lo reinicia tras la espera y rechaza lo que llega mientras', async () => {
    const h = harness();
    await h.engine.request(WINDOW_REQUEST);
    await crash(h);
    expect(h.engine.state).toBe('waiting');
    expect(h.timer.pending).toEqual([1000]);
    await expect(h.engine.request(WINDOW_REQUEST)).rejects.toBeInstanceOf(EngineUnavailableError);
    h.timer.advance(1000);
    await flush();
    expect(h.launcher.processes).toHaveLength(2);
    expect(h.engine.state).toBe('running');
  });

  it('se rinde con cinco caidas seguidas y Reintentar vuelve a empezar', async () => {
    const h = harness();
    await h.engine.request(WINDOW_REQUEST);
    for (const delay of [1000, 2000, 4000, 8000]) {
      await crash(h);
      h.timer.advance(delay);
      await flush();
    }
    await crash(h);
    expect(h.engine.state).toBe('failed');
    expect(h.giveUps).toHaveLength(1);
    expect(h.launcher.processes).toHaveLength(5);
    await expect(h.engine.request(WINDOW_REQUEST)).rejects.toBeInstanceOf(EngineUnavailableError);
    h.giveUps[0]?.();
    await flush();
    expect(h.engine.state).toBe('running');
    expect(h.launcher.processes).toHaveLength(6);
  });

  it('un motor con otro protocolo no se reintenta y se avisa una vez', async () => {
    const h = harness();
    h.launcher.responder = infoResponder(2);
    await expect(h.engine.request(WINDOW_REQUEST)).rejects.toBeInstanceOf(EngineUnavailableError);
    expect(h.engine.state).toBe('failed');
    expect(h.launcher.last.stopped).toBe(true);
    expect(h.mismatches).toEqual([{ outcome: 'mismatch', engineVersion: 2, expected: 1 }]);
    h.launcher.last.exit();
    await flush();
    expect(h.timer.pending).toEqual([]);
    expect(h.giveUps).toHaveLength(0);
  });

  it('un fallo al lanzar el binario cuenta como caida', async () => {
    const h = harness();
    h.launcher.failNext = true;
    await expect(h.engine.request(WINDOW_REQUEST)).rejects.toBeInstanceOf(EngineUnavailableError);
    expect(h.engine.state).toBe('waiting');
    expect(h.logger.messages('error')[0]).toContain('could not launch the engine');
  });

  it('si engine/info falla, para el proceso y su salida pasa por los reintentos', async () => {
    const h = harness();
    h.launcher.responder = () => Promise.reject(new EngineUnavailableError('broken pipe'));
    await expect(h.engine.request(WINDOW_REQUEST)).rejects.toBeInstanceOf(EngineUnavailableError);
    expect(h.launcher.last.stopped).toBe(true);
    await crash(h);
    expect(h.timer.pending).toEqual([1000]);
  });

  it('una caida durante el arranque no deja la peticion colgada', async () => {
    const h = harness();
    let rejectInfo: (error: Error) => void = () => undefined;
    h.launcher.responder = () =>
      new Promise((_resolve, reject) => {
        rejectInfo = reject;
      });
    const pending = h.engine.request(WINDOW_REQUEST);
    await flush();
    h.launcher.last.exit({ code: null, signal: 'SIGKILL' });
    // La conexion real rechaza lo pendiente al cerrarse el proceso.
    rejectInfo(new EngineUnavailableError('the engine stopped'));
    await expect(pending).rejects.toBeInstanceOf(EngineUnavailableError);
    expect(h.logger.messages('warn')).toEqual(['engine exited: signal SIGKILL']);
    expect(h.engine.state).toBe('waiting');
  });

  it('reenvia las notificaciones de cualquier proceso, tambien tras reiniciar', async () => {
    const h = harness();
    const received: RpcNotification[] = [];
    const unsubscribe = h.engine.onNotification((notification) => received.push(notification));
    await h.engine.request(WINDOW_REQUEST);
    const changed: RpcNotification = { jsonrpc: '2.0', method: 'repo/changed', params: {} };
    h.launcher.last.connection.emit(changed);
    await crash(h);
    h.timer.advance(1000);
    await flush();
    h.launcher.last.connection.emit(changed);
    unsubscribe();
    h.launcher.last.connection.emit(changed);
    expect(received).toEqual([changed, changed]);
  });

  it('parar cancela el reinicio pendiente y para el proceso en marcha', async () => {
    const h = harness();
    await h.engine.request(WINDOW_REQUEST);
    await crash(h);
    h.engine.stop();
    expect(h.timer.pending).toEqual([]);
    expect(h.engine.state).toBe('stopped');

    const running = harness();
    await running.engine.request(WINDOW_REQUEST);
    running.engine.stop();
    expect(running.launcher.last.stopped).toBe(true);
    running.launcher.last.exit({ code: 0, signal: null });
    await flush();
    expect(running.timer.pending).toEqual([]);
    await expect(running.engine.request(WINDOW_REQUEST)).rejects.toBeInstanceOf(
      EngineUnavailableError,
    );
  });

  it('reintentar sin haberse rendido no hace nada', async () => {
    const h = harness();
    await h.engine.request(WINDOW_REQUEST);
    h.engine.retry();
    expect(h.launcher.processes).toHaveLength(1);
  });
});
