import type { AccountSummary, RepoContext, RepoSummary, WorkspaceSummary } from '../repos/model';

/**
 * Una version reducida de los repos, cuentas y workspaces del prototipo, con los
 * casos que importan: repos sin remoto, una cuenta borrada, un workspace borrado
 * y un repo en dos workspaces.
 */

export const ACCOUNTS: readonly AccountSummary[] = [
  { id: 'a1', platform: 'github', user: 'marta-dev', host: 'github.com' },
  { id: 'a2', platform: 'gitlab', user: 'mruiz', host: 'gitlab.empresa.com' },
  { id: 'a3', platform: 'gitea', user: 'marta', host: 'git.casa.lan' },
];

export const WORKSPACES: readonly WorkspaceSummary[] = [
  { id: 'w1', name: 'Producto' },
  { id: 'w2', name: 'Backend' },
];

function repo(overrides: Partial<RepoSummary> & Pick<RepoSummary, 'id' | 'name'>): RepoSummary {
  return {
    alias: null,
    path: `~/code/${overrides.name}`,
    branch: 'main',
    hasRemote: true,
    accountId: null,
    workspaceIds: [],
    ...overrides,
  };
}

export const REPOS: readonly RepoSummary[] = [
  repo({ id: 'r1', name: 'ramal-desktop', accountId: 'a1', workspaceIds: ['w1', 'w2'] }),
  repo({ id: 'r2', name: 'api-gateway', accountId: 'a2', workspaceIds: ['w2'], branch: 'develop' }),
  repo({ id: 'r3', name: 'notas', hasRemote: false, workspaceIds: ['w-borrado'] }),
  repo({ id: 'r4', name: 'dotfiles', accountId: 'a3', alias: 'config', branch: 'master' }),
  repo({ id: 'r5', name: 'huerfano', accountId: 'a-borrada', workspaceIds: ['w1'] }),
];

export const CONTEXT: RepoContext = { accounts: ACCOUNTS, workspaces: WORKSPACES };
