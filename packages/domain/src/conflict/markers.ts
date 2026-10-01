import type { ConflictLabels, ConflictSegment } from './model';

/**
 * Marcadores de conflicto de git: deteccion en un texto editado a mano y parser
 * del texto con marcadores a segmentos (la inversa de buildOutput).
 *
 * Se reconoce el marcador del ancestro (`|||||||`, de diff3 y zdiff3), que el
 * prototipo ignoraba, y una linea `=======` solo cuenta dentro de un bloque
 * abierto: fuera es un subrayado de Markdown o RST legitimo (gaps B-21).
 */

export type MarkerKind = 'start' | 'base' | 'separator' | 'end';

export interface MarkerLine {
  /** Numero de linea empezando en 1, como lo ensena un editor. */
  readonly line: number;
  readonly kind: MarkerKind;
}

interface Marker {
  readonly kind: MarkerKind;
  readonly label: string;
}

const PATTERNS: readonly (readonly [MarkerKind, RegExp])[] = [
  ['start', /^<{7}(?: (.*))?$/u],
  ['base', /^\|{7}(?: (.*))?$/u],
  ['separator', /^={7}$/u],
  ['end', /^>{7}(?: (.*))?$/u],
];

/** Un fichero con fin de linea CRLF deja un \r al final de cada linea al partir por \n. */
function markerOf(line: string): Marker | null {
  const clean = line.endsWith('\r') ? line.slice(0, -1) : line;
  for (const [kind, pattern] of PATTERNS) {
    const match = pattern.exec(clean);
    if (match !== null) return { kind, label: match[1] ?? '' };
  }
  return null;
}

export function findConflictMarkers(text: string): MarkerLine[] {
  const found: MarkerLine[] = [];
  let open = false;
  for (const [index, line] of text.split('\n').entries()) {
    const marker = markerOf(line);
    if (marker === null || (marker.kind === 'separator' && !open)) continue;
    if (marker.kind === 'start') open = true;
    if (marker.kind === 'end') open = false;
    found.push({ line: index + 1, kind: marker.kind });
  }
  return found;
}

export function hasConflictMarkers(text: string): boolean {
  return findConflictMarkers(text).length > 0;
}

export type ParseConflictResult =
  | {
      readonly ok: true;
      readonly segments: readonly ConflictSegment[];
      readonly labels: ConflictLabels | null;
    }
  | {
      readonly ok: false;
      readonly error: 'unterminated' | 'unexpected-marker';
      readonly line: number;
    };

type State = 'context' | 'ours' | 'base' | 'theirs';

interface MutableLabels {
  ours: string;
  theirs: string;
  base?: string;
}

/**
 * Maquina de estados del parser. Es una clase para que el estado mutable quede
 * encerrado en un solo sitio; desde fuera solo se ve parseConflictMarkers.
 */
class MarkerParser {
  private state: State = 'context';
  private context: string[] = [];
  private ours: string[] = [];
  private base: string[] | null = null;
  private theirs: string[] = [];
  private pendingOurs = '';
  private pendingBase: string | undefined = undefined;
  readonly segments: ConflictSegment[] = [];
  labels: MutableLabels | null = null;

  get open(): boolean {
    return this.state !== 'context';
  }

  /** Devuelve false si la linea es un marcador fuera de sitio. */
  feed(line: string): boolean {
    const marker = markerOf(line);
    if (marker === null || this.isPlainInContext(marker)) {
      this.append(line);
      return true;
    }
    return this.apply(marker);
  }

  finish(): void {
    this.flushContext();
  }

  private isPlainInContext(marker: Marker): boolean {
    return this.state === 'context' && (marker.kind === 'separator' || marker.kind === 'base');
  }

  private append(line: string): void {
    if (this.state === 'context') this.context.push(line);
    else if (this.state === 'theirs') this.theirs.push(line);
    else (this.base ?? this.ours).push(line);
  }

  private apply(marker: Marker): boolean {
    const { state } = this;
    if (marker.kind === 'start' && state === 'context') this.openHunk(marker.label);
    else if (marker.kind === 'base' && state === 'ours') this.openBase(marker.label);
    else if (marker.kind === 'separator' && (state === 'ours' || state === 'base')) {
      this.state = 'theirs';
    } else if (marker.kind === 'end' && state === 'theirs') this.closeHunk(marker.label);
    else return false;
    return true;
  }

  private flushContext(): void {
    if (this.context.length === 0) return;
    this.segments.push({ kind: 'context', lines: this.context });
    this.context = [];
  }

  private openHunk(label: string): void {
    this.flushContext();
    this.state = 'ours';
    this.ours = [];
    this.base = null;
    this.theirs = [];
    this.pendingOurs = label;
    this.pendingBase = undefined;
  }

  private openBase(label: string): void {
    this.state = 'base';
    this.base = [];
    this.pendingBase = label;
  }

  private closeHunk(label: string): void {
    const { base, ours, theirs } = this;
    this.segments.push(
      base === null ? { kind: 'hunk', ours, theirs } : { kind: 'hunk', ours, theirs, base },
    );
    if (this.labels === null) {
      this.labels = { ours: this.pendingOurs, theirs: label };
      if (this.pendingBase !== undefined) this.labels.base = this.pendingBase;
    }
    this.state = 'context';
  }
}

/** Texto con marcadores a segmentos. Un bloque sin cerrar o un marcador fuera de sitio es un error. */
export function parseConflictMarkers(text: string): ParseConflictResult {
  const parser = new MarkerParser();
  const lines = text.split('\n');
  for (const [index, line] of lines.entries()) {
    if (!parser.feed(line)) return { ok: false, error: 'unexpected-marker', line: index + 1 };
  }
  if (parser.open) return { ok: false, error: 'unterminated', line: lines.length };
  parser.finish();
  return { ok: true, segments: parser.segments, labels: parser.labels };
}
