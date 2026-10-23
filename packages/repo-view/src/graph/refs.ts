import type { RefLabel } from '@git-rudder/bridge';
import { type IconName, cssVar, laneColor } from '@git-rudder/ui';

/** Pildoras visibles por fila; el resto se resume en "+n" (diseno de la vista de repositorio). */
export const MAX_VISIBLE_REFS = 2;

export interface SplitRefs {
  readonly shown: readonly RefLabel[];
  readonly hidden: readonly RefLabel[];
}

/** Parte las refs de una fila en las que se pintan y las que van en "+n". */
export function splitRefs(refs: readonly RefLabel[], max: number = MAX_VISIBLE_REFS): SplitRefs {
  return { shown: refs.slice(0, max), hidden: refs.slice(max) };
}

/**
 * Icono de cada pildora: el del prototipo (comprobado para HEAD, flechas para
 * una rama remota, rombo para un tag, rama para una local) traducido a nombres
 * semanticos de @git-rudder/ui. El juego concreto (Lucide o Codicons) lo pone el host.
 */
export function refIcon(ref: RefLabel): IconName {
  if (ref.isHead) return 'check';
  switch (ref.kind) {
    case 'remoteBranch':
      return 'remote';
    case 'tag':
      return 'tag';
    case 'localBranch':
      return 'branch';
  }
}

/**
 * Color de la pildora: los tags van en el tono de aviso y las ramas en el color
 * del carril de su commit, como en el prototipo.
 */
export function refColor(ref: RefLabel, lane: number): string {
  return ref.kind === 'tag' ? cssVar('--r-warn') : laneColor(lane);
}
