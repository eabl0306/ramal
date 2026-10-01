import type {
  ConflictFile,
  ConflictLabels,
  ConflictSide,
  FileResolution,
  HunkChoice,
  HunkSegment,
} from './model';

/**
 * El panel "Resultado": como quedara el fichero con las elecciones actuales.
 *
 * Orden fijo dentro de cada bloque: lineas elegidas de A y despues de B, como en
 * el prototipo. Toda linea elegida sale, aunque el otro lado tenga otra igual:
 * un `}`, una linea en blanco o un `enabled: true` repetidos son codigo de cada
 * lado, y quitarlos dejaba el fichero roto contando como resuelto. Las lineas
 * comunes del principio y del final las saca git con marcadores zdiff3, que pide
 * el motor (gaps B-19); si aun asi quedan iguales en los dos lados, `duplicates`
 * las cuenta para que la interfaz avise, sin tocar el texto.
 */

export type OutputLine =
  | { readonly kind: 'context'; readonly text: string; readonly number: number }
  | {
      readonly kind: 'picked';
      readonly side: ConflictSide;
      readonly hunk: number;
      readonly line: number;
      readonly text: string;
      readonly number: number;
    }
  | { readonly kind: 'manual'; readonly text: string; readonly number: number }
  | { readonly kind: 'unresolved'; readonly hunk: number };

export interface ConflictOutput {
  readonly lines: readonly OutputLine[];
  /** Texto final. Los bloques sin resolver llevan sus marcadores: es la semilla del editor manual. */
  readonly text: string;
  /** Lineas elegidas de B con otra identica elegida de A en el mismo bloque. Es un aviso. */
  readonly duplicates: number;
  readonly unresolved: number;
}

function marker(prefix: string, label: string | undefined): string {
  return label === undefined || label === '' ? prefix : `${prefix} ${label}`;
}

/** El bloque tal y como lo escribe git, para dejarlo sin resolver en el texto. */
export function hunkMarkerText(hunk: HunkSegment, labels: ConflictLabels): string[] {
  const base = hunk.base === undefined ? [] : [marker('|||||||', labels.base), ...hunk.base];
  return [
    marker('<<<<<<<', labels.ours),
    ...hunk.ours,
    ...base,
    '=======',
    ...hunk.theirs,
    marker('>>>>>>>', labels.theirs),
  ];
}

/**
 * Acumula las lineas del resultado. Es una clase para que los contadores
 * mutables (numero de linea, repetidas, sin resolver) vivan en un solo sitio.
 */
class OutputBuilder {
  readonly lines: OutputLine[] = [];
  private readonly text: string[] = [];
  private number = 0;
  duplicates = 0;
  unresolved = 0;

  private readonly labels: ConflictLabels;

  constructor(labels: ConflictLabels) {
    this.labels = labels;
  }

  context(text: string): void {
    this.number += 1;
    this.lines.push({ kind: 'context', text, number: this.number });
    this.text.push(text);
  }

  hunk(hunk: HunkSegment, choice: HunkChoice | undefined, index: number): void {
    if (choice === undefined || choice.decision === 'none') {
      this.lines.push({ kind: 'unresolved', hunk: index });
      this.text.push(...hunkMarkerText(hunk, this.labels));
      this.unresolved += 1;
      return;
    }
    if (choice.decision === 'lines') this.picked(hunk, choice, index);
  }

  result(): ConflictOutput {
    const { lines, duplicates, unresolved } = this;
    return { lines, text: this.text.join('\n'), duplicates, unresolved };
  }

  private pick(side: ConflictSide, hunk: number, line: number, text: string): void {
    this.number += 1;
    this.lines.push({ kind: 'picked', side, hunk, line, text, number: this.number });
    this.text.push(text);
  }

  private picked(hunk: HunkSegment, choice: HunkChoice, index: number): void {
    const pickedOurs = new Map<string, number>();
    hunk.ours.forEach((text, line) => {
      if (choice.ours[line] !== true) return;
      pickedOurs.set(text, (pickedOurs.get(text) ?? 0) + 1);
      this.pick('ours', index, line, text);
    });
    hunk.theirs.forEach((text, line) => {
      if (choice.theirs[line] !== true) return;
      this.pick('theirs', index, line, text);
      const remaining = pickedOurs.get(text) ?? 0;
      // Una linea en blanco repetida no es una eleccion duplicada: avisar de ella es ruido.
      if (remaining === 0 || text.trim() === '') return;
      pickedOurs.set(text, remaining - 1);
      this.duplicates += 1;
    });
  }
}

function manualOutput(manual: string): ConflictOutput {
  const lines = manual
    .split('\n')
    .map((text, index): OutputLine => ({ kind: 'manual', text, number: index + 1 }));
  return { lines, text: manual, duplicates: 0, unresolved: 0 };
}

export function buildOutput(
  file: ConflictFile,
  resolution: FileResolution,
  labels: ConflictLabels,
): ConflictOutput {
  if (resolution.manual !== null) return manualOutput(resolution.manual);
  const builder = new OutputBuilder(labels);
  let index = 0;
  for (const segment of file.segments) {
    if (segment.kind === 'context') {
      segment.lines.forEach((text) => {
        builder.context(text);
      });
      continue;
    }
    builder.hunk(segment, resolution.hunks[index], index);
    index += 1;
  }
  return builder.result();
}
