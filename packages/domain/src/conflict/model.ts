/**
 * Modelo del resolvedor de conflictos interactivo.
 *
 * El motor entrega cada fichero ya partido en segmentos (texto comun y bloques
 * en conflicto); la interfaz guarda aqui que lineas de cada lado se quedan y
 * construye el texto final. El motor comprueba que no quedan marcadores antes de
 * completar el merge: esta validacion es comodidad, la suya manda.
 */

export type ConflictSide = 'ours' | 'theirs';

export interface ContextSegment {
  readonly kind: 'context';
  readonly lines: readonly string[];
}

export interface HunkSegment {
  readonly kind: 'hunk';
  readonly ours: readonly string[];
  readonly theirs: readonly string[];
  /** Lineas del ancestro comun, si git escribio marcadores diff3 o zdiff3. */
  readonly base?: readonly string[];
}

export type ConflictSegment = ContextSegment | HunkSegment;

export interface ConflictFile {
  readonly path: string;
  readonly segments: readonly ConflictSegment[];
}

/** Lo que va detras de los marcadores al reconstruir el texto con conflictos. */
export interface ConflictLabels {
  readonly ours: string;
  readonly theirs: string;
  readonly base?: string;
}

/**
 * Decision explicita por bloque (gaps B-20). `empty` es "este bloque se queda en
 * nada", que el prototipo no podia expresar y que cuenta como resuelto.
 */
export type HunkDecision = 'none' | 'lines' | 'empty';

export interface HunkChoice {
  readonly decision: HunkDecision;
  /** Una casilla por linea de cada lado: si esa linea va al resultado. */
  readonly ours: readonly boolean[];
  readonly theirs: readonly boolean[];
}

export interface FileResolution {
  readonly hunks: readonly HunkChoice[];
  /** Texto editado a mano y guardado. Si existe, manda sobre las elecciones. */
  readonly manual: string | null;
  /**
   * Borrador del editor manual, que sobrevive a cambiar de fichero (gaps B-22).
   * null cuando no se esta editando.
   */
  readonly draft: string | null;
  /**
   * "Marcar como resuelto igualmente" (gaps B-21): el texto manual tiene algo
   * que parece un marcador, por ejemplo un subrayado de Markdown, y quien edita
   * dice que es legitimo.
   */
  readonly acceptMarkers: boolean;
}

export function hunksOf(file: ConflictFile): HunkSegment[] {
  return file.segments.filter((segment): segment is HunkSegment => segment.kind === 'hunk');
}

function emptyChoice(hunk: HunkSegment): HunkChoice {
  return {
    decision: 'none',
    ours: hunk.ours.map(() => false),
    theirs: hunk.theirs.map(() => false),
  };
}

/** Estado inicial: nada elegido, como en el prototipo. */
export function initResolution(file: ConflictFile): FileResolution {
  return { hunks: hunksOf(file).map(emptyChoice), manual: null, draft: null, acceptMarkers: false };
}
