import { PLATFORM_IDS, type PlatformId } from '../platforms/platforms';

import { type RepoContext, type RepoSummary, repoAccount, repoWorkspaces } from './model';

/**
 * Agrupacion de la lista de repositorios. Los grupos llevan una clave, no un
 * nombre: "Sin workspace" o "Solo local" los traduce la interfaz. Los grupos
 * vacios no se devuelven.
 */

export type RepoGrouping = 'none' | 'workspace' | 'platform' | 'account';

export type RepoGroupKey =
  | { readonly kind: 'all' }
  | { readonly kind: 'workspace'; readonly id: string }
  | { readonly kind: 'no-workspace' }
  | { readonly kind: 'platform'; readonly id: PlatformId }
  | { readonly kind: 'account'; readonly id: string }
  | { readonly kind: 'local' };

export interface RepoGroup {
  readonly key: RepoGroupKey;
  readonly repos: readonly RepoSummary[];
}

function nonEmpty(groups: readonly RepoGroup[]): RepoGroup[] {
  return groups.filter((group) => group.repos.length > 0);
}

/** Un repo aparece en el grupo de cada workspace al que pertenece. */
function byWorkspace(repos: readonly RepoSummary[], context: RepoContext): RepoGroup[] {
  const groups = context.workspaces.map((workspace): RepoGroup => ({
    key: { kind: 'workspace', id: workspace.id },
    repos: repos.filter((repo) => repo.workspaceIds.includes(workspace.id)),
  }));
  const orphans = repos.filter((repo) => repoWorkspaces(repo, context.workspaces).length === 0);
  return nonEmpty([...groups, { key: { kind: 'no-workspace' }, repos: orphans }]);
}

function byPlatform(repos: readonly RepoSummary[], context: RepoContext): RepoGroup[] {
  const platformOf = (repo: RepoSummary): PlatformId | null =>
    repoAccount(repo, context.accounts)?.platform ?? null;
  const groups = PLATFORM_IDS.map((id): RepoGroup => ({
    key: { kind: 'platform', id },
    repos: repos.filter((repo) => platformOf(repo) === id),
  }));
  const local = repos.filter((repo) => platformOf(repo) === null);
  return nonEmpty([...groups, { key: { kind: 'local' }, repos: local }]);
}

function byAccount(repos: readonly RepoSummary[], context: RepoContext): RepoGroup[] {
  const groups = context.accounts.map((account): RepoGroup => ({
    key: { kind: 'account', id: account.id },
    repos: repos.filter((repo) => repo.accountId === account.id),
  }));
  const local = repos.filter((repo) => repoAccount(repo, context.accounts) === null);
  return nonEmpty([...groups, { key: { kind: 'local' }, repos: local }]);
}

export function groupRepos(
  repos: readonly RepoSummary[],
  grouping: RepoGrouping,
  context: RepoContext,
): RepoGroup[] {
  switch (grouping) {
    case 'workspace':
      return byWorkspace(repos, context);
    case 'platform':
      return byPlatform(repos, context);
    case 'account':
      return byAccount(repos, context);
    case 'none':
      return nonEmpty([{ key: { kind: 'all' }, repos }]);
  }
}
