import { PLATFORMS } from '../platforms/platforms';

import {
  type RepoContext,
  type RepoFilters,
  type RepoSummary,
  repoAccount,
  repoWorkspaces,
} from './model';

/**
 * Filtros de la lista de repositorios. La busqueda mira cada campo por separado:
 * el prototipo unia todo con espacios y "main gitlab" podia casar con trozos de
 * dos campos distintos.
 */

function searchableFields(repo: RepoSummary, context: RepoContext): string[] {
  const account = repoAccount(repo, context.accounts);
  const fields = [repo.name, repo.alias, repo.path, repo.branch];
  if (account !== null) {
    fields.push(account.user, account.host, PLATFORMS[account.platform].label);
  }
  return fields.filter((field): field is string => field !== null && field !== '');
}

function matchesQuery(repo: RepoSummary, query: string, context: RepoContext): boolean {
  const needle = query.trim().toLowerCase();
  if (needle === '') return true;
  return searchableFields(repo, context).some((field) => field.toLowerCase().includes(needle));
}

function matchesKind(repo: RepoSummary, filters: RepoFilters): boolean {
  if (filters.kind === 'local') return !repo.hasRemote;
  if (filters.kind === 'remote') return repo.hasRemote;
  return true;
}

function matchesPlatforms(repo: RepoSummary, filters: RepoFilters, context: RepoContext): boolean {
  if (filters.platforms.length === 0) return true;
  const account = repoAccount(repo, context.accounts);
  return account !== null && filters.platforms.includes(account.platform);
}

function matchesWorkspace(repo: RepoSummary, filters: RepoFilters, context: RepoContext): boolean {
  const { workspace } = filters;
  if (workspace.kind === 'any') return true;
  if (workspace.kind === 'none') return repoWorkspaces(repo, context.workspaces).length === 0;
  return repo.workspaceIds.includes(workspace.id);
}

export function repoMatchesFilters(
  repo: RepoSummary,
  filters: RepoFilters,
  context: RepoContext,
): boolean {
  return (
    matchesKind(repo, filters) &&
    matchesPlatforms(repo, filters, context) &&
    (filters.accountId === null || repo.accountId === filters.accountId) &&
    matchesWorkspace(repo, filters, context) &&
    matchesQuery(repo, filters.query, context)
  );
}

export function filterRepos(
  repos: readonly RepoSummary[],
  filters: RepoFilters,
  context: RepoContext,
): RepoSummary[] {
  return repos.filter((repo) => repoMatchesFilters(repo, filters, context));
}

/** El contador de la barra lateral de cada workspace. */
export function countReposInWorkspace(repos: readonly RepoSummary[], workspaceId: string): number {
  return repos.filter((repo) => repo.workspaceIds.includes(workspaceId)).length;
}

/** "Usada en N repos" de una cuenta. */
export function countReposUsingAccount(repos: readonly RepoSummary[], accountId: string): number {
  return repos.filter((repo) => repo.accountId === accountId).length;
}
