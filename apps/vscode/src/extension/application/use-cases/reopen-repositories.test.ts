import { createTranslator } from '@ramal/i18n';
import { describe, expect, it } from 'vitest';

import type { IdentifiedRpcRequest } from '../../../shared/messages';
import { FakeEngine, fail, ok } from '../../../test/engine-fakes';
import { FakeNotifications, FakePanels, RecordingLogger } from '../../../test/host-fakes';
import { EngineUnavailableError } from '../engine-errors';
import { createRequestIds } from '../request-ids';

import { type ReopenRepositoriesDeps, reopenRepositories } from './reopen-repositories';

/** Contesta `repo/open` con un id que sale de la ruta: `/work/a` da `id-a`. */
function openByPath(request: IdentifiedRpcRequest): Promise<ReturnType<typeof ok>> {
  const { path } = request.params as { readonly path: string };
  const handle = { repoId: `id-${path.slice(-1)}`, root: path, isDetached: false, isEmpty: false };
  return Promise.resolve(ok(request, handle));
}

function deps(roots: readonly string[]): ReopenRepositoriesDeps & {
  readonly panels: FakePanels;
  readonly notifications: FakeNotifications;
} {
  const panels = new FakePanels();
  for (const root of roots) {
    panels.show({ surface: 'graph', repoId: 'repo-old', root, title: 'Grafo de Ramal' });
  }
  return {
    ids: createRequestIds(),
    panels,
    notifications: new FakeNotifications(),
    translate: createTranslator('es'),
    logger: new RecordingLogger(),
  };
}

describe('volver a abrir los repositorios tras reiniciar el motor', () => {
  it('abre la raiz de cada panel en el proceso nuevo y les pasa el id que da', async () => {
    const d = deps(['/work/a', '/work/b', '/work/a']);
    const engine = new FakeEngine();
    engine.respond = openByPath;
    await reopenRepositories(engine, d);
    expect(engine.requests.map((request) => request.params)).toEqual([
      { path: '/work/a' },
      { path: '/work/b' },
    ]);
    expect(d.panels.rebound).toEqual([
      { root: '/work/a', repoId: 'id-a' },
      { root: '/work/b', repoId: 'id-b' },
    ]);
    expect(d.notifications.errors).toEqual([]);
  });

  it('sin paneles abiertos no llama al motor', async () => {
    const engine = new FakeEngine();
    await reopenRepositories(engine, deps([]));
    expect(engine.requests).toEqual([]);
  });

  it('cierra el panel de una carpeta que ya no es un repositorio y lo avisa', async () => {
    const d = deps(['/work/gone', '/work/b']);
    const engine = new FakeEngine();
    engine.respond = (request) =>
      (request.params as { readonly path: string }).path === '/work/gone'
        ? Promise.resolve(fail(request, 'not_a_repository'))
        : openByPath(request);
    await reopenRepositories(engine, d);
    expect(d.panels.closed).toEqual(['/work/gone']);
    expect(d.panels.rebound).toEqual([{ root: '/work/b', repoId: 'id-b' }]);
    expect(d.notifications.errors).toEqual([
      { message: 'La carpeta no es un repositorio de Git.', actions: [] },
    ]);
  });

  it('si el motor se vuelve a caer a mitad no cierra nada: lo reabre el siguiente arranque', async () => {
    const d = deps(['/work/a']);
    const engine = new FakeEngine();
    engine.respond = () => Promise.reject(new EngineUnavailableError('the engine stopped'));
    await reopenRepositories(engine, d);
    expect(d.panels.closed).toEqual([]);
    expect(d.panels.rebound).toEqual([]);
    expect(d.notifications.errors).toEqual([]);
  });
});
