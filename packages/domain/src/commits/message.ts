/**
 * Reglas del mensaje de commit: el contador del resumen, cuando se puede
 * hacer commit y como se parte y se compone el mensaje completo.
 */

/** La convencion de git: un resumen que cabe en una linea de `git log --oneline`. */
export const SUMMARY_LIMIT = 72;

const graphemes = new Intl.Segmenter(undefined, { granularity: 'grapheme' });

/**
 * Caracteres que ve una persona, no unidades UTF-16 (gaps B-09): una tilde
 * compuesta o un emoji con modificadores cuentan uno.
 */
export function countGraphemes(text: string): number {
  let count = 0;
  for (const _ of graphemes.segment(text)) count += 1;
  return count;
}

export interface SummaryCounter {
  readonly length: number;
  /** Puede ser negativo: el contador se pinta en rojo pero no corta lo escrito. */
  readonly remaining: number;
  readonly over: boolean;
}

export function summaryCounter(summary: string, limit: number = SUMMARY_LIMIT): SummaryCounter {
  const length = countGraphemes(summary);
  return { length, remaining: limit - length, over: length > limit };
}

export interface CommitReadiness {
  readonly summary: string;
  readonly stagedCount: number;
  /** amend no necesita ficheros preparados: puede cambiar solo el mensaje. */
  readonly amend: boolean;
}

export function canCommit(input: CommitReadiness): boolean {
  return (input.stagedCount > 0 || input.amend) && input.summary.trim() !== '';
}

export interface CommitMessageParts {
  readonly summary: string;
  readonly body: string;
}

/**
 * Parte un mensaje en resumen (primera linea) y cuerpo. El prototipo de VS Code
 * se quedaba con la primera linea y perdia el cuerpo (gaps B-07).
 */
export function splitCommitMessage(message: string): CommitMessageParts {
  const normalized = message.replaceAll('\r\n', '\n');
  const newline = normalized.indexOf('\n');
  if (newline === -1) return { summary: normalized.trim(), body: '' };
  return {
    summary: normalized.slice(0, newline).trim(),
    body: normalized.slice(newline + 1).trim(),
  };
}

/** Resumen, linea en blanco y cuerpo, como lo espera git. Sin cuerpo, solo el resumen. */
export function composeCommitMessage(parts: CommitMessageParts): string {
  const summary = parts.summary.trim();
  const body = parts.body.trim();
  return body === '' ? summary : `${summary}\n\n${body}`;
}
