import type { RebasePlan } from './plan';

/**
 * El fichero todo de `git rebase -i` que corresponde al plan. La linea de arriba
 * se aplica primero, igual que en la interfaz. El asunto es solo informativo
 * para git; los mensajes de `reword` viajan aparte.
 *
 * Los `drop` se escriben explicitos: quitar la linea tambien descarta el commit,
 * pero falla con `rebase.missingCommitsCheck=error`.
 */
export function rebaseTodo(plan: RebasePlan): string {
  const lines = plan.items.map((item) => {
    const subject = item.subject.split('\n', 1)[0] ?? '';
    return `${item.action} ${item.sha} ${subject}`.trimEnd();
  });
  return lines.length === 0 ? '' : `${lines.join('\n')}\n`;
}
