import type { RepoHandle } from '@ramal/bridge';
import { describe, expect, it } from 'vitest';

import {
  INITIAL_WORKSPACE,
  type WorkspaceAction,
  type WorkspaceState,
  activeTab,
  repoLabel,
  workspaceReducer,
} from './workspace';

function handle(repoId: string, root: string): RepoHandle {
  return { repoId, root, headRef: 'main', isDetached: false, isEmpty: false };
}

function run(actions: readonly WorkspaceAction[], from = INITIAL_WORKSPACE): WorkspaceState {
  return actions.reduce(workspaceReducer, from);
}

function opened(repo: RepoHandle): WorkspaceAction {
  return { type: 'pick-finished', result: { outcome: 'opened', repo } };
}

const ramal = handle('repo-1', '/home/ana/code/ramal');
const engine = handle('repo-2', 'C:\\Users\\ana\\engine\\');

describe('workspaceReducer: abrir un repositorio', () => {
  it('marca que espera al dialogo y borra el error anterior', () => {
    const failed: WorkspaceState = { ...INITIAL_WORKSPACE, openError: 'git_not_found' };

    const state = workspaceReducer(failed, { type: 'pick-started' });

    expect(state.picking).toBe(true);
    expect(state.openError).toBeNull();
  });

  it('abre una pestana con el nombre de la carpeta y la pone delante', () => {
    const state = run([{ type: 'pick-started' }, opened(ramal)]);

    expect(state.picking).toBe(false);
    expect(state.tabs).toEqual([{ repoId: 'repo-1', root: ramal.root, label: 'ramal' }]);
    expect(activeTab(state)?.repoId).toBe('repo-1');
  });

  it('no duplica la pestana si el motor devuelve un repositorio ya abierto', () => {
    const state = run([opened(ramal), opened(engine), opened(ramal)]);

    expect(state.tabs.map((tab) => tab.repoId)).toEqual(['repo-1', 'repo-2']);
    expect(state.active).toBe('repo-1');
  });

  it('se queda donde estaba si el usuario cancela', () => {
    const before = run([opened(ramal), { type: 'tab-activated', repoId: null }]);

    const state = run(
      [{ type: 'pick-started' }, { type: 'pick-finished', result: { outcome: 'cancelled' } }],
      before,
    );

    expect(state).toEqual(before);
  });

  it('guarda el tipo de error para ensenarlo en Inicio', () => {
    const state = run([
      { type: 'pick-started' },
      { type: 'pick-finished', result: { outcome: 'failed', kind: 'not_a_repository' } },
    ]);

    expect(state.openError).toBe('not_a_repository');
    expect(state.picking).toBe(false);
    expect(state.active).toBeNull();
  });
});

describe('workspaceReducer: pestanas', () => {
  it('va a Inicio y vuelve a una pestana abierta', () => {
    const home = run([opened(ramal), { type: 'tab-activated', repoId: null }]);
    expect(activeTab(home)).toBeNull();

    const back = workspaceReducer(home, { type: 'tab-activated', repoId: 'repo-1' });
    expect(activeTab(back)?.label).toBe('ramal');
  });

  it('ignora activar una pestana que no existe', () => {
    const state = run([opened(ramal), { type: 'tab-activated', repoId: 'repo-9' }]);

    expect(state.active).toBe('repo-1');
  });

  it('al cerrar la pestana visible pasa a la ultima que queda', () => {
    const state = run([opened(ramal), opened(engine), { type: 'tab-closed', repoId: 'repo-2' }]);

    expect(state.tabs.map((tab) => tab.repoId)).toEqual(['repo-1']);
    expect(state.active).toBe('repo-1');
  });

  it('al cerrar la ultima pestana vuelve a Inicio', () => {
    const state = run([opened(ramal), { type: 'tab-closed', repoId: 'repo-1' }]);

    expect(state.tabs).toEqual([]);
    expect(state.active).toBeNull();
  });

  it('cerrar una pestana de fondo no cambia la visible', () => {
    const state = run([opened(ramal), opened(engine), { type: 'tab-closed', repoId: 'repo-1' }]);

    expect(state.active).toBe('repo-2');
  });
});

describe('repoLabel', () => {
  it.each([
    ['/home/ana/code/ramal', 'ramal'],
    ['/home/ana/code/ramal/', 'ramal'],
    ['C:\\Users\\ana\\engine\\', 'engine'],
    ['/', '/'],
  ])('%s se ensena como %s', (root, label) => {
    expect(repoLabel(root)).toBe(label);
  });
});
