import { describe, expect, it } from 'vitest';

import { CONTEXT, REPOS } from '../test/repo-fixtures';

import {
  countReposInWorkspace,
  countReposUsingAccount,
  filterRepos,
  repoMatchesFilters,
} from './filter';
import { groupRepos } from './group';
import { EMPTY_REPO_FILTERS, type RepoFilters, hasActiveRepoFilters } from './model';

function ids(filters: Partial<RepoFilters>): string[] {
  return filterRepos(REPOS, { ...EMPTY_REPO_FILTERS, ...filters }, CONTEXT).map((repo) => repo.id);
}

describe('filterRepos', () => {
  it('sin filtros devuelve todo', () => {
    expect(ids({})).toEqual(['r1', 'r2', 'r3', 'r4', 'r5']);
  });

  it('local y remoto salen de si el repo tiene remoto, no de la cuenta', () => {
    expect(ids({ kind: 'local' })).toEqual(['r3']);
    expect(ids({ kind: 'remote' })).toEqual(['r1', 'r2', 'r4', 'r5']);
  });

  it('las plataformas filtran por la cuenta existente, en seleccion multiple', () => {
    expect(ids({ platforms: ['gitlab'] })).toEqual(['r2']);
    expect(ids({ platforms: ['github', 'gitea'] })).toEqual(['r1', 'r4']);
  });

  it('filtra por cuenta exacta', () => {
    expect(ids({ accountId: 'a1' })).toEqual(['r1']);
  });

  it('sin workspace ignora los ids de workspaces borrados', () => {
    expect(ids({ workspace: { kind: 'none' } })).toEqual(['r3', 'r4']);
    expect(ids({ workspace: { kind: 'workspace', id: 'w2' } })).toEqual(['r1', 'r2']);
  });

  it('busca en cada campo por separado, sin distinguir mayusculas', () => {
    expect(ids({ query: '  GITLAB ' })).toEqual(['r2']);
    expect(ids({ query: 'config' })).toEqual(['r4']);
    expect(ids({ query: 'develop' })).toEqual(['r2']);
    expect(ids({ query: 'casa.lan' })).toEqual(['r4']);
    expect(ids({ query: 'main gitlab' })).toEqual([]);
  });

  it('combina filtros', () => {
    const filters = { ...EMPTY_REPO_FILTERS, kind: 'remote' as const, query: 'git-rudder' };
    expect(repoMatchesFilters(REPOS[0]!, filters, CONTEXT)).toBe(true);
    expect(repoMatchesFilters(REPOS[2]!, filters, CONTEXT)).toBe(false);
  });
});

describe('hasActiveRepoFilters', () => {
  it('solo cuenta los filtros, no una busqueda en blanco', () => {
    expect(hasActiveRepoFilters(EMPTY_REPO_FILTERS)).toBe(false);
    expect(hasActiveRepoFilters({ ...EMPTY_REPO_FILTERS, query: '   ' })).toBe(false);
    expect(hasActiveRepoFilters({ ...EMPTY_REPO_FILTERS, platforms: ['gitea'] })).toBe(true);
    expect(hasActiveRepoFilters({ ...EMPTY_REPO_FILTERS, workspace: { kind: 'none' } })).toBe(true);
  });
});

describe('groupRepos', () => {
  const keys = (grouping: Parameters<typeof groupRepos>[1]): unknown[] =>
    groupRepos(REPOS, grouping, CONTEXT).map((group) => [
      group.key,
      group.repos.map((repo) => repo.id),
    ]);

  it('sin agrupar devuelve un solo grupo', () => {
    expect(keys('none')).toEqual([[{ kind: 'all' }, ['r1', 'r2', 'r3', 'r4', 'r5']]]);
    expect(groupRepos([], 'none', CONTEXT)).toEqual([]);
  });

  it('por workspace repite el repo en cada uno y termina con los que no tienen', () => {
    expect(keys('workspace')).toEqual([
      [{ kind: 'workspace', id: 'w1' }, ['r1', 'r5']],
      [{ kind: 'workspace', id: 'w2' }, ['r1', 'r2']],
      [{ kind: 'no-workspace' }, ['r3', 'r4']],
    ]);
  });

  it('por plataforma sigue el orden de la tabla y oculta los grupos vacios', () => {
    expect(keys('platform')).toEqual([
      [{ kind: 'platform', id: 'github' }, ['r1']],
      [{ kind: 'platform', id: 'gitlab' }, ['r2']],
      [{ kind: 'platform', id: 'gitea' }, ['r4']],
      [{ kind: 'local' }, ['r3', 'r5']],
    ]);
  });

  it('por cuenta sigue el orden de las cuentas', () => {
    expect(keys('account')).toEqual([
      [{ kind: 'account', id: 'a1' }, ['r1']],
      [{ kind: 'account', id: 'a2' }, ['r2']],
      [{ kind: 'account', id: 'a3' }, ['r4']],
      [{ kind: 'local' }, ['r3', 'r5']],
    ]);
  });
});

describe('contadores', () => {
  it('cuenta repos por workspace y por cuenta', () => {
    expect(countReposInWorkspace(REPOS, 'w1')).toBe(2);
    expect(countReposUsingAccount(REPOS, 'a2')).toBe(1);
  });
});
