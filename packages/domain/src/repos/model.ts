import type { PlatformId } from '../platforms/platforms';

/**
 * Lo minimo de un repositorio, una cuenta y un workspace que necesitan los
 * filtros y la agrupacion de la lista de repos. Cada app adapta sus datos a esto.
 */

export interface RepoSummary {
  readonly id: string;
  readonly name: string;
  readonly alias: string | null;
  readonly path: string | null;
  readonly branch: string | null;
  /**
   * Si el repo tiene un remoto en git. El filtro Local / Remoto sale de aqui y no
   * de si hay una cuenta enlazada, que era el error del prototipo (gaps B-05).
   */
  readonly hasRemote: boolean;
  readonly accountId: string | null;
  readonly workspaceIds: readonly string[];
}

export interface AccountSummary {
  readonly id: string;
  readonly platform: PlatformId;
  readonly user: string;
  readonly host: string;
}

export interface WorkspaceSummary {
  readonly id: string;
  readonly name: string;
}

export interface RepoContext {
  readonly accounts: readonly AccountSummary[];
  readonly workspaces: readonly WorkspaceSummary[];
}

export type RepoKindFilter = 'all' | 'local' | 'remote';

export type WorkspaceFilter =
  | { readonly kind: 'any' }
  | { readonly kind: 'none' }
  | { readonly kind: 'workspace'; readonly id: string };

export interface RepoFilters {
  readonly query: string;
  readonly kind: RepoKindFilter;
  /** Seleccion multiple; vacia es "todas". */
  readonly platforms: readonly PlatformId[];
  readonly accountId: string | null;
  readonly workspace: WorkspaceFilter;
}

export const EMPTY_REPO_FILTERS: RepoFilters = {
  query: '',
  kind: 'all',
  platforms: [],
  accountId: null,
  workspace: { kind: 'any' },
};

/** Si "Limpiar filtros" tiene algo que limpiar. La agrupacion no es un filtro. */
export function hasActiveRepoFilters(filters: RepoFilters): boolean {
  return (
    filters.query.trim() !== '' ||
    filters.kind !== 'all' ||
    filters.platforms.length > 0 ||
    filters.accountId !== null ||
    filters.workspace.kind !== 'any'
  );
}

/** La cuenta del repo, si existe todavia: un id de una cuenta borrada no cuenta. */
export function repoAccount(
  repo: RepoSummary,
  accounts: readonly AccountSummary[],
): AccountSummary | null {
  if (repo.accountId === null) return null;
  return accounts.find((account) => account.id === repo.accountId) ?? null;
}

/** Los workspaces del repo que siguen existiendo, en el orden de la lista de workspaces. */
export function repoWorkspaces(
  repo: RepoSummary,
  workspaces: readonly WorkspaceSummary[],
): WorkspaceSummary[] {
  return workspaces.filter((workspace) => repo.workspaceIds.includes(workspace.id));
}
