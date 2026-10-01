/**
 * Modelo del editor de rebase interactivo: la lista de commits (el mas antiguo
 * primero, que es el que se aplica primero, igual que el todo de git) con la
 * accion de cada uno. El motor vuelve a validar el plan antes de ejecutarlo.
 */

export const REBASE_ACTIONS = ['pick', 'reword', 'edit', 'squash', 'fixup', 'drop'] as const;

export type RebaseAction = (typeof REBASE_ACTIONS)[number];

export interface RebaseCommit {
  readonly sha: string;
  readonly subject: string;
}

export interface RebaseItem {
  readonly sha: string;
  readonly subject: string;
  readonly action: RebaseAction;
  /** Mensaje nuevo; solo se usa con `reword`. Empieza siendo el asunto actual. */
  readonly message: string;
}

export interface RebasePlan {
  /** Rama o sha sobre el que se reconstruye. */
  readonly onto: string;
  readonly items: readonly RebaseItem[];
}

/** Todo empieza en `pick`, como en `git rebase -i`. */
export function createRebasePlan(onto: string, commits: readonly RebaseCommit[]): RebasePlan {
  return {
    onto,
    items: commits.map((commit) => ({
      sha: commit.sha,
      subject: commit.subject,
      action: 'pick',
      message: commit.subject,
    })),
  };
}

function updateItem(
  plan: RebasePlan,
  index: number,
  update: (item: RebaseItem) => RebaseItem,
): RebasePlan {
  if (index < 0 || index >= plan.items.length) return plan;
  return { ...plan, items: plan.items.map((item, at) => (at === index ? update(item) : item)) };
}

export function setRebaseAction(plan: RebasePlan, index: number, action: RebaseAction): RebasePlan {
  return updateItem(plan, index, (item) => ({ ...item, action }));
}

export function setRebaseMessage(plan: RebasePlan, index: number, message: string): RebasePlan {
  return updateItem(plan, index, (item) => ({ ...item, message }));
}

/**
 * Mueve un commit de `from` a `to`. Las flechas son mover a la posicion de al
 * lado; arrastrar es mover a la posicion donde se suelta. Fuera de rango no hace
 * nada.
 */
export function moveRebaseItem(plan: RebasePlan, from: number, to: number): RebasePlan {
  const count = plan.items.length;
  if (from < 0 || from >= count || to < 0 || to >= count || from === to) return plan;
  const items = [...plan.items];
  const [moved] = items.splice(from, 1);
  if (moved === undefined) return plan;
  items.splice(to, 0, moved);
  return { ...plan, items };
}

export interface RebaseSummary {
  /** pick, reword y edit: commits que se conservan como tales. */
  readonly pick: number;
  /** squash y fixup: commits que se funden con el anterior. */
  readonly squash: number;
  readonly drop: number;
}

export function summarizeRebasePlan(plan: RebasePlan): RebaseSummary {
  const count = (actions: readonly RebaseAction[]): number =>
    plan.items.filter((item) => actions.includes(item.action)).length;
  return {
    pick: count(['pick', 'reword', 'edit']),
    squash: count(['squash', 'fixup']),
    drop: count(['drop']),
  };
}
