import type { RebasePlan } from './plan';

/**
 * Validacion inmediata del plan, para que el boton "Iniciar rebase" diga que
 * falta antes de llamar al motor. Los errores bloquean; los avisos se ensenan y
 * dejan seguir.
 */

export type RebaseIssue =
  | { readonly code: 'empty'; readonly severity: 'error' }
  | {
      readonly code: 'needs-previous-commit';
      readonly severity: 'error';
      readonly index: number;
      readonly action: 'squash' | 'fixup';
    }
  | { readonly code: 'reword-without-message'; readonly severity: 'error'; readonly index: number }
  | { readonly code: 'dirty-tree'; readonly severity: 'error' }
  | { readonly code: 'all-dropped'; readonly severity: 'warning' }
  | { readonly code: 'needs-force-push'; readonly severity: 'warning' };

export interface RebaseContext {
  /** Hay cambios sin commit en el arbol de trabajo. */
  readonly dirtyTree?: boolean;
  /** Se guardaran en un stash antes y se recuperaran despues. */
  readonly autostash?: boolean;
  /** Algun commit del plan ya esta en el remoto: hara falta force push con lease. */
  readonly pushed?: boolean;
}

/**
 * git no puede fundir el primer commit que se aplica con "el anterior", porque
 * no lo hay: "cannot 'squash' without a previous commit". Los `drop` del
 * principio no cuentan como anterior.
 */
function firstKeptIssue(plan: RebasePlan): RebaseIssue | null {
  const index = plan.items.findIndex((item) => item.action !== 'drop');
  const first = plan.items[index];
  if (first === undefined) return null;
  if (first.action !== 'squash' && first.action !== 'fixup') return null;
  return { code: 'needs-previous-commit', severity: 'error', index, action: first.action };
}

function rewordIssues(plan: RebasePlan): RebaseIssue[] {
  const issues: RebaseIssue[] = [];
  plan.items.forEach((item, index) => {
    if (item.action === 'reword' && item.message.trim() === '') {
      issues.push({ code: 'reword-without-message', severity: 'error', index });
    }
  });
  return issues;
}

function contextIssues(plan: RebasePlan, context: RebaseContext): RebaseIssue[] {
  const issues: RebaseIssue[] = [];
  if (context.dirtyTree === true && context.autostash !== true) {
    issues.push({ code: 'dirty-tree', severity: 'error' });
  }
  if (plan.items.every((item) => item.action === 'drop')) {
    issues.push({ code: 'all-dropped', severity: 'warning' });
  }
  if (context.pushed === true) issues.push({ code: 'needs-force-push', severity: 'warning' });
  return issues;
}

export function validateRebasePlan(plan: RebasePlan, context: RebaseContext = {}): RebaseIssue[] {
  if (plan.items.length === 0) return [{ code: 'empty', severity: 'error' }];
  const first = firstKeptIssue(plan);
  return [
    ...(first === null ? [] : [first]),
    ...rewordIssues(plan),
    ...contextIssues(plan, context),
  ];
}

export function canStartRebase(issues: readonly RebaseIssue[]): boolean {
  return issues.every((issue) => issue.severity !== 'error');
}
