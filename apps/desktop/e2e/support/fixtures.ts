import type { GraphEdge, GraphRow, RefLabel, RepoHandle } from '@ramal/bridge';

import type { E2eScenario } from './scenario';

export const REPO: RepoHandle = {
  repoId: 'repo-1',
  root: '/home/ana/code/ramal',
  headRef: 'main',
  headSha: sha(0),
  isDetached: false,
  isEmpty: false,
};

function sha(index: number): string {
  return String(index).padStart(40, '0');
}

function edge(
  parentRow: number,
  lanes: readonly [number, number, number],
  kind: GraphEdge['kind'],
): GraphEdge {
  const [fromLane, travelLane, toLane] = lanes;
  return { parentSha: sha(parentRow), parentRow, fromLane, travelLane, toLane, kind };
}

function ref(name: string, kind: RefLabel['kind'], isHead = false): RefLabel {
  return { name, fullName: name, kind, isHead };
}

function row(index: number, summary: string, extra: Partial<GraphRow>): GraphRow {
  return {
    sha: sha(index),
    shortSha: sha(index).slice(0, 7),
    parents: [],
    lane: 0,
    edges: [],
    summary,
    authorName: 'Ana Ruiz',
    authorEmail: 'ana@example.com',
    authoredAt: '2026-09-30T10:00:00+02:00',
    refs: [],
    isHead: false,
    ...extra,
  };
}

/**
 * Un historial pequeno con una rama que se fusiona: dos carriles, una arista de
 * merge en tres tramos y pildoras de rama local, remota y tag.
 */
export const HISTORY: readonly GraphRow[] = [
  row(0, "Merge branch 'feature/grafo'", {
    parents: [sha(1), sha(2)],
    edges: [edge(1, [0, 0, 0], 'firstParent'), edge(2, [0, 1, 1], 'merge')],
    refs: [ref('main', 'localBranch', true), ref('origin/main', 'remoteBranch')],
    isHead: true,
  }),
  row(1, 'Pinta el grafo con el motor', {
    parents: [sha(3)],
    edges: [edge(3, [0, 0, 0], 'firstParent')],
  }),
  row(2, 'Abre la carpeta desde Rust', {
    lane: 1,
    parents: [sha(3)],
    edges: [edge(3, [1, 1, 0], 'firstParent')],
    refs: [ref('feature/grafo', 'localBranch')],
  }),
  row(3, 'Primer commit', { refs: [ref('v0.1.0', 'tag')] }),
];

export const OPENS_REPO: E2eScenario = { pick: { outcome: 'opened', repo: REPO }, rows: HISTORY };
