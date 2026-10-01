/**
 * Conventional Commits: `tipo(ambito)!: asunto`. Lo usa la preferencia de IA
 * "Usar Conventional Commits" y el aviso de la caja de commit.
 */

export const CONVENTIONAL_TYPES = [
  'feat',
  'fix',
  'docs',
  'style',
  'refactor',
  'perf',
  'test',
  'build',
  'ci',
  'chore',
  'revert',
] as const;

export interface ConventionalSummary {
  readonly type: string;
  readonly scope: string | null;
  readonly breaking: boolean;
  readonly subject: string;
}

const PATTERN = /^(?<type>[a-z]+)(?:\((?<scope>[^()\s]+)\))?(?<breaking>!)?: (?<subject>.*)$/u;

export type ConventionalCheck =
  | { readonly ok: true; readonly parsed: ConventionalSummary }
  | { readonly ok: false; readonly reason: 'format' | 'unknown-type' | 'empty-subject' };

export function parseConventional(summary: string): ConventionalSummary | null {
  const groups = PATTERN.exec(summary.trimStart())?.groups;
  if (groups === undefined) return null;
  return {
    type: groups['type'] ?? '',
    scope: groups['scope'] ?? null,
    breaking: groups['breaking'] === '!',
    subject: (groups['subject'] ?? '').trim(),
  };
}

export function checkConventional(
  summary: string,
  types: readonly string[] = CONVENTIONAL_TYPES,
): ConventionalCheck {
  const parsed = parseConventional(summary);
  if (parsed === null) return { ok: false, reason: 'format' };
  if (!types.includes(parsed.type)) return { ok: false, reason: 'unknown-type' };
  if (parsed.subject === '') return { ok: false, reason: 'empty-subject' };
  return { ok: true, parsed };
}

export function formatConventional(parts: ConventionalSummary): string {
  const scope = parts.scope === null || parts.scope === '' ? '' : `(${parts.scope})`;
  return `${parts.type}${scope}${parts.breaking ? '!' : ''}: ${parts.subject.trim()}`;
}
