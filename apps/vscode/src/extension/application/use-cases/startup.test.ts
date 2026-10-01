import { PROTOCOL_VERSION } from '@ramal/bridge';
import { describe, expect, it } from 'vitest';

import { FakeLauncher, fail, infoResponder } from '../../../test/engine-fakes';
import {
  FakeDialogs,
  FakeNotifications,
  FakePanels,
  FakeTimer,
  RecordingLogger,
  fakeChannel,
  flush,
} from '../../../test/host-fakes';
import { EngineUnavailableError } from '../engine-errors';

import { createHostProbe } from './host-probe';
import { openGraph } from './open-graph';
import { createWebviewRouter } from './route-webview-message';
import { startup } from './startup';

const REQUEST = { jsonrpc: '2.0', id: 50, method: 'graph/window', params: {} } as const;

function boot(language: string): {
  readonly runtime: ReturnType<typeof startup>;
  readonly launcher: FakeLauncher;
  readonly timer: FakeTimer;
  readonly notifications: FakeNotifications;
} {
  const launcher = new FakeLauncher();
  const timer = new FakeTimer();
  const notifications = new FakeNotifications();
  const runtime = startup({
    language,
    launcher,
    timer,
    notifications,
    logger: new RecordingLogger(),
  });
  return { runtime, launcher, timer, notifications };
}

describe('arranque del host', () => {
  it('sigue el idioma de VS Code con la regla comun', () => {
    expect(boot('es-ES').runtime.locale).toBe('es');
    expect(boot('es-419').runtime.locale).toBe('es');
    expect(boot('fr').runtime.locale).toBe('en');
    expect(boot('es').runtime.translate('vscode.tab.graph')).toBe('Grafo de Ramal');
  });

  it('no lanza el motor al arrancar y compara el protocolo generado al lanzarlo', async () => {
    const { runtime, launcher } = boot('en');
    expect(launcher.processes).toHaveLength(0);
    expect(runtime.protocolVersion).toBe(PROTOCOL_VERSION);
    await runtime.engine.request(REQUEST);
    expect(launcher.last.connection.requests[0]?.method).toBe('engine/info');
  });

  it('avisa de un motor con otro protocolo como instalacion danada', async () => {
    const { runtime, launcher, notifications } = boot('es');
    launcher.responder = infoResponder(PROTOCOL_VERSION + 1);
    await expect(runtime.engine.request(REQUEST)).rejects.toBeInstanceOf(EngineUnavailableError);
    expect(notifications.errors[0]?.message).toContain(
      `usa la versión ${String(PROTOCOL_VERSION + 1)} del protocolo`,
    );
  });

  it('al rendirse ensena Reintentar y pulsarlo vuelve a arrancar el motor', async () => {
    const { runtime, launcher, timer, notifications } = boot('es');
    notifications.reply = 'Reintentar';
    await runtime.engine.request(REQUEST);
    for (const delay of [1000, 2000, 4000, 8000]) {
      launcher.last.exit();
      await flush();
      timer.advance(delay);
      await flush();
    }
    launcher.last.exit();
    await flush();
    expect(notifications.errors).toEqual([
      {
        message: 'El motor de Ramal se ha detenido después de caerse varias veces.',
        actions: ['Reintentar'],
      },
    ]);
    expect(runtime.engine.state).toBe('running');
    expect(launcher.processes).toHaveLength(6);
  });

  it('sin git, Open Graph dice que falta git y el motor no entra en bucle de reinicios', async () => {
    const { runtime, launcher, timer, notifications } = boot('es');
    launcher.responder = (request) => Promise.resolve(fail(request, 'git_not_found'));
    const result = await openGraph({
      engine: runtime.engine,
      ids: runtime.ids,
      repositories: { list: () => [{ name: 'ramal', path: '/work/ramal' }] },
      dialogs: new FakeDialogs(),
      notifications,
      panels: new FakePanels(),
      translate: runtime.translate,
      logger: new RecordingLogger(),
    });
    expect(result).toEqual({ outcome: 'failed', kind: 'git_not_found' });
    expect(notifications.errors).toEqual([
      { message: 'No se encuentra Git en este equipo.', actions: [] },
    ]);
    launcher.last.exit();
    await flush();
    expect(timer.pending).toEqual([]);
    expect(launcher.processes).toHaveLength(1);
  });
});

describe('sonda de los tests de integracion', () => {
  it('guarda lo enrutado y llama al motor como el host', async () => {
    const { runtime } = boot('en');
    const router = createWebviewRouter({
      engine: runtime.engine,
      ids: runtime.ids,
      hostRequests: { handle: () => Promise.resolve(null) },
      logger: new RecordingLogger(),
      locale: runtime.locale,
      protocolVersion: runtime.protocolVersion,
    });
    const probe = createHostProbe(router, runtime.engine, runtime.ids);
    router.attach(fakeChannel('panel-1'));
    await router.receive('panel-1', { type: 'rpc/request', request: REQUEST });
    expect(probe.routed()).toEqual([
      { panelId: 'panel-1', method: 'graph/window', outcome: 'answered' },
    ]);
    const info = await probe.call('engine/info', {});
    expect(info.protocolVersion).toBe(PROTOCOL_VERSION);
  });
});
