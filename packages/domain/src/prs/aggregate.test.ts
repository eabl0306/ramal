import { describe, expect, it } from 'vitest';

import { prNumberLabel } from '../platforms/platforms';
import { REPOS } from '../test/repo-fixtures';

import {
  type PrState,
  type PullRequestSummary,
  countOpenPrsInRepo,
  countPrsByState,
  derivePrState,
  filterPrsByState,
  isOpenPr,
  sortPrsByUpdate,
  summarizeWorkspacePrs,
} from './aggregate';

const PRS: readonly PullRequestSummary[] = [
  { id: 'p1', repoId: 'r1', number: 41, state: 'open', updatedAt: '2026-09-29T10:00:00Z' },
  { id: 'p2', repoId: 'r1', number: 42, state: 'draft', updatedAt: '2026-09-30T08:00:00Z' },
  { id: 'p3', repoId: 'r2', number: 7, state: 'merged', updatedAt: '2026-09-28T12:00:00Z' },
  { id: 'p4', repoId: 'r2', number: 8, state: 'changes', updatedAt: '2026-09-30T09:00:00Z' },
  { id: 'p5', repoId: 'r4', number: 3, state: 'closed', updatedAt: '2026-09-01T00:00:00Z' },
];

describe('agregacion de pull requests', () => {
  it('cuenta por estado con el total en all', () => {
    expect(countPrsByState(PRS)).toEqual({
      all: 5,
      open: 1,
      draft: 1,
      approved: 0,
      changes: 1,
      merged: 1,
      closed: 1,
    });
  });

  it('abierto incluye borrador, aprobado y cambios solicitados', () => {
    const open: PrState[] = ['open', 'draft', 'approved', 'changes'];
    expect(open.every(isOpenPr)).toBe(true);
    expect(isOpenPr('merged')).toBe(false);
    expect(countOpenPrsInRepo(PRS, 'r2')).toBe(1);
  });

  it('filtra por estado y all devuelve una copia de todo', () => {
    expect(filterPrsByState(PRS, 'draft').map((pr) => pr.id)).toEqual(['p2']);
    const all = filterPrsByState(PRS, 'all');
    expect(all).toEqual(PRS);
    expect(all).not.toBe(PRS);
  });

  it('ordena por la ultima actualizacion', () => {
    expect(sortPrsByUpdate(PRS).map((pr) => pr.id)).toEqual(['p4', 'p2', 'p1', 'p3', 'p5']);
  });

  it('resume los PRs de los repos de un workspace', () => {
    expect(summarizeWorkspacePrs('w2', REPOS, PRS)).toEqual({
      repoCount: 2,
      prCount: 4,
      openCount: 3,
      counts: { all: 4, open: 1, draft: 1, approved: 0, changes: 1, merged: 1, closed: 0 },
    });
  });
});

describe('derivePrState', () => {
  const base = { merged: false, closed: false, draft: false, reviewDecision: null };

  it('lo cerrado manda sobre todo lo demas', () => {
    expect(derivePrState({ ...base, merged: true, draft: true })).toBe('merged');
    expect(derivePrState({ ...base, closed: true, reviewDecision: 'approved' })).toBe('closed');
  });

  it('despues el borrador y despues la revision', () => {
    expect(derivePrState({ ...base, draft: true, reviewDecision: 'approved' })).toBe('draft');
    expect(derivePrState({ ...base, reviewDecision: 'changes-requested' })).toBe('changes');
    expect(derivePrState({ ...base, reviewDecision: 'approved' })).toBe('approved');
    expect(derivePrState({ ...base, reviewDecision: 'review-required' })).toBe('open');
  });
});

describe('prNumberLabel', () => {
  it('GitLab usa ! y el resto #', () => {
    expect(prNumberLabel('gitlab', 12)).toBe('!12');
    expect(prNumberLabel('github', 12)).toBe('#12');
  });
});
