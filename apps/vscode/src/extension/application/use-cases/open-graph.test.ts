import { createTranslator } from '@ramal/i18n';
import { describe, expect, it } from 'vitest';

import { FakeEngine, fail, ok } from '../../../test/engine-fakes';
import {
  FakeDialogs,
  FakeNotifications,
  FakePanels,
  RecordingLogger,
} from '../../../test/host-fakes';
import { EngineUnavailableError } from '../engine-errors';
import type { WorkspaceFolderInfo } from '../ports/workspace-repositories';
import { createRequestIds } from '../request-ids';

import { type OpenGraphDeps, openGraph } from './open-graph';

const HANDLE = {
  repoId: 'repo-1',
  root: '/work/ramal',
  headRef: 'main',
  headSha: 'a'.repeat(40),
  isDetached: false,
  isEmpty: false,
};

function deps(folders: readonly WorkspaceFolderInfo[]): OpenGraphDeps & {
  readonly engine: FakeEngine;
  readonly dialogs: FakeDialogs;
  readonly notifications: FakeNotifications;
  readonly panels: FakePanels;
} {
  const engine = new FakeEngine();
  engine.respond = (request) => Promise.resolve(ok(request, HANDLE));
  return {
    engine,
    ids: createRequestIds(),
    repositories: { list: () => folders },
    dialogs: new FakeDialogs(),
    notifications: new FakeNotifications(),
    panels: new FakePanels(),
    translate: createTranslator('es'),
    logger: new RecordingLogger(),
  };
}

const RAMAL = { name: 'ramal', path: '/work/ramal' };
const API = { name: 'api', path: '/work/api' };

describe('Ramal: Open Graph', () => {
  it('sin carpetas en el workspace avisa y no abre nada', async () => {
    const d = deps([]);
    expect(await openGraph(d)).toEqual({ outcome: 'no-folder' });
    expect(d.notifications.infos).toEqual([
      'Abre una carpeta con un repositorio Git para ver su grafo.',
    ]);
    expect(d.engine.requests).toEqual([]);
  });

  it('con una carpeta, el host abre el repo y ensena el panel con su repoId', async () => {
    const d = deps([RAMAL]);
    expect(await openGraph(d)).toEqual({ outcome: 'opened', repoId: 'repo-1' });
    expect(d.engine.requests).toEqual([
      { jsonrpc: '2.0', id: 1, method: 'repo/open', params: { path: '/work/ramal' } },
    ]);
    expect(d.panels.shown).toEqual([
      { surface: 'graph', repoId: 'repo-1', root: '/work/ramal', title: 'Grafo de Ramal' },
    ]);
    expect(d.dialogs.shown).toEqual([]);
  });

  it('con varias carpetas pregunta cual con un QuickPick', async () => {
    const d = deps([RAMAL, API]);
    d.dialogs.choose = 1;
    await openGraph(d);
    expect(d.dialogs.shown).toEqual([
      { labels: ['ramal', 'api'], placeholder: 'Elige el repositorio que quieres abrir' },
    ]);
    expect(d.engine.requests[0]?.params).toEqual({ path: '/work/api' });
  });

  it('si se cancela el QuickPick no hace nada', async () => {
    const d = deps([RAMAL, API]);
    d.dialogs.choose = null;
    expect(await openGraph(d)).toEqual({ outcome: 'cancelled' });
    expect(d.engine.requests).toEqual([]);
  });

  it('una carpeta que no es un repositorio se avisa con el texto de su tipo', async () => {
    const d = deps([RAMAL]);
    d.engine.respond = (request) => Promise.resolve(fail(request, 'not_a_repository'));
    expect(await openGraph(d)).toEqual({ outcome: 'failed', kind: 'not_a_repository' });
    expect(d.notifications.errors).toEqual([
      { message: 'La carpeta no es un repositorio de Git.', actions: [] },
    ]);
    expect(d.panels.shown).toEqual([]);
  });

  it('si el motor no esta, lo dice asi', async () => {
    const d = deps([RAMAL]);
    d.engine.respond = () => Promise.reject(new EngineUnavailableError('gone'));
    expect(await openGraph(d)).toEqual({ outcome: 'failed', kind: 'engine_unavailable' });
  });
});
