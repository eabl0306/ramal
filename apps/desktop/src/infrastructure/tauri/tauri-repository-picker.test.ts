import type { RepoHandle } from '@ramal/bridge';
import { describe, expect, it, vi } from 'vitest';

import type { Invoke } from './invoke';
import {
  PICK_AND_OPEN_COMMAND,
  createTauriRepositoryPicker,
  errorKind,
} from './tauri-repository-picker';

const repo: RepoHandle = {
  repoId: 'repo-1',
  root: '/home/ana/code/ramal',
  headRef: 'main',
  headSha: '1'.repeat(40),
  isDetached: false,
  isEmpty: false,
};

function answering(result: Promise<unknown>): Invoke {
  return vi.fn(() => result) as unknown as Invoke;
}

describe('createTauriRepositoryPicker', () => {
  it('devuelve el repositorio abierto por el host', async () => {
    const invoke = answering(Promise.resolve(repo));

    await expect(createTauriRepositoryPicker(invoke).pickAndOpen()).resolves.toEqual({
      outcome: 'opened',
      repo,
    });
    expect(invoke).toHaveBeenCalledWith(PICK_AND_OPEN_COMMAND);
  });

  it('null es que el usuario cancelo el dialogo', async () => {
    const picker = createTauriRepositoryPicker(answering(Promise.resolve(null)));

    await expect(picker.pickAndOpen()).resolves.toEqual({ outcome: 'cancelled' });
  });

  it('un RpcError del motor da su kind', async () => {
    const rejection = {
      code: -32_000,
      message: 'not a repository',
      data: { kind: 'not_a_repository' },
    };
    // eslint-disable-next-line @typescript-eslint/prefer-promise-reject-errors -- Tauri rechaza invoke con el RpcError serializado: un objeto plano, no un Error
    const picker = createTauriRepositoryPicker(answering(Promise.reject(rejection)));

    await expect(picker.pickAndOpen()).resolves.toEqual({
      outcome: 'failed',
      kind: 'not_a_repository',
    });
  });
});

describe('errorKind', () => {
  it.each([
    [new Error('ipc closed'), 'transport'],
    ['command rpc not allowed', 'transport'],
    [null, 'transport'],
    [{ code: -32_000 }, 'transport'],
    [{ data: null }, 'transport'],
    [{ data: { reason: 'x' } }, 'transport'],
    [{ data: { kind: 'git_not_found' } }, 'git_not_found'],
    [{ data: { kind: 'something_new' } }, 'internal'],
  ])('%j da %s', (error, kind) => {
    expect(errorKind(error)).toBe(kind);
  });
});
