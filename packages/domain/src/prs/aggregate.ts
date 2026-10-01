import type { RepoSummary } from '../repos/model';

/**
 * Pull requests agregados por estado: los chips con contador de la vista de un
 * workspace, el subtitulo "N PRs abiertos" y el estado que se pinta a partir de
 * lo que devuelve cada plataforma.
 */

export const PR_STATES = ['open', 'draft', 'approved', 'changes', 'merged', 'closed'] as const;

export type PrState = (typeof PR_STATES)[number];

export type PrStateFilter = PrState | 'all';

/** Abierto en sentido amplio: todo lo que aun puede fusionarse. */
export const OPEN_PR_STATES: readonly PrState[] = ['open', 'draft', 'approved', 'changes'];

export interface PullRequestSummary {
  readonly id: string;
  readonly repoId: string;
  readonly number: number;
  readonly state: PrState;
  /** Instante ISO 8601 de la ultima actualizacion. */
  readonly updatedAt: string;
}

export function isOpenPr(state: PrState): boolean {
  return OPEN_PR_STATES.includes(state);
}

export function countPrsByState(prs: readonly PullRequestSummary[]): Record<PrStateFilter, number> {
  const counts: Record<PrStateFilter, number> = {
    all: prs.length,
    open: 0,
    draft: 0,
    approved: 0,
    changes: 0,
    merged: 0,
    closed: 0,
  };
  for (const pr of prs) counts[pr.state] += 1;
  return counts;
}

export function filterPrsByState<T extends PullRequestSummary>(
  prs: readonly T[],
  filter: PrStateFilter,
): T[] {
  return filter === 'all' ? [...prs] : prs.filter((pr) => pr.state === filter);
}

/** Lo ultimo actualizado primero. El prototipo no ordenaba. */
export function sortPrsByUpdate<T extends PullRequestSummary>(prs: readonly T[]): T[] {
  return [...prs].sort((a, b) => Date.parse(b.updatedAt) - Date.parse(a.updatedAt));
}

export interface WorkspacePrSummary {
  readonly repoCount: number;
  readonly prCount: number;
  readonly openCount: number;
  readonly counts: Record<PrStateFilter, number>;
}

export function summarizeWorkspacePrs(
  workspaceId: string,
  repos: readonly RepoSummary[],
  prs: readonly PullRequestSummary[],
): WorkspacePrSummary {
  const repoIds = new Set(
    repos.filter((repo) => repo.workspaceIds.includes(workspaceId)).map((repo) => repo.id),
  );
  const own = prs.filter((pr) => repoIds.has(pr.repoId));
  return {
    repoCount: repoIds.size,
    prCount: own.length,
    openCount: own.filter((pr) => isOpenPr(pr.state)).length,
    counts: countPrsByState(own),
  };
}

export function countOpenPrsInRepo(prs: readonly PullRequestSummary[], repoId: string): number {
  return prs.filter((pr) => pr.repoId === repoId && isOpenPr(pr.state)).length;
}

export type ReviewDecision = 'approved' | 'changes-requested' | 'review-required' | null;

export interface PlatformPrStatus {
  readonly merged: boolean;
  readonly closed: boolean;
  readonly draft: boolean;
  readonly reviewDecision: ReviewDecision;
}

/** El estado de interfaz a partir de los datos de la plataforma: lo cerrado manda, luego borrador, luego revision. */
export function derivePrState(status: PlatformPrStatus): PrState {
  if (status.merged) return 'merged';
  if (status.closed) return 'closed';
  if (status.draft) return 'draft';
  if (status.reviewDecision === 'changes-requested') return 'changes';
  if (status.reviewDecision === 'approved') return 'approved';
  return 'open';
}
