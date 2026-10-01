/**
 * Que es cada lado segun la operacion que dejo el conflicto (gaps B-23). El
 * prototipo siempre decia "A = tu rama, B = entrante", que en un rebase es al
 * reves de lo que parece: "ours" es la base sobre la que se reconstruye y
 * "theirs" el commit que se esta reaplicando. La interfaz traduce el papel.
 */

export type ConflictOperation = 'merge' | 'rebase' | 'cherry-pick' | 'revert' | 'stash-apply';

export type ConflictSideRole =
  | 'current-branch'
  | 'incoming-branch'
  | 'rebase-base'
  | 'replayed-commit'
  | 'picked-commit'
  | 'reverted-commit'
  | 'stash';

export interface ConflictSideRoles {
  readonly ours: ConflictSideRole;
  readonly theirs: ConflictSideRole;
}

const ROLES: Readonly<Record<ConflictOperation, ConflictSideRoles>> = {
  merge: { ours: 'current-branch', theirs: 'incoming-branch' },
  rebase: { ours: 'rebase-base', theirs: 'replayed-commit' },
  'cherry-pick': { ours: 'current-branch', theirs: 'picked-commit' },
  revert: { ours: 'current-branch', theirs: 'reverted-commit' },
  'stash-apply': { ours: 'current-branch', theirs: 'stash' },
};

export function conflictSideRoles(operation: ConflictOperation): ConflictSideRoles {
  return ROLES[operation];
}
