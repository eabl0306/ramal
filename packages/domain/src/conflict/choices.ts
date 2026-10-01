import {
  type ConflictFile,
  type ConflictSide,
  type FileResolution,
  type HunkChoice,
  hunksOf,
} from './model';

/**
 * Operaciones sobre las elecciones de un fichero. Todas son inmutables y todas
 * descartan la edicion manual y su borrador, porque estos se escribieron sobre
 * las elecciones anteriores. La interfaz pregunta antes de llamarlas cuando
 * `hasManualEdit` es verdadero (gaps B-22).
 */

function decisionFor(ours: readonly boolean[], theirs: readonly boolean[]): HunkChoice['decision'] {
  return ours.includes(true) || theirs.includes(true) ? 'lines' : 'none';
}

function withHunk(
  resolution: FileResolution,
  hunkIndex: number,
  update: (choice: HunkChoice) => HunkChoice,
): FileResolution {
  const current = resolution.hunks[hunkIndex];
  if (current === undefined) return resolution;
  const hunks = resolution.hunks.map((choice, index) =>
    index === hunkIndex ? update(choice) : choice,
  );
  return { hunks, manual: null, draft: null, acceptMarkers: false };
}

function setSide(choice: HunkChoice, side: ConflictSide, values: readonly boolean[]): HunkChoice {
  const ours = side === 'ours' ? values : choice.ours;
  const theirs = side === 'theirs' ? values : choice.theirs;
  return { decision: decisionFor(ours, theirs), ours, theirs };
}

/** El "+" o "-" de una linea, en su panel o en el resultado. */
export function toggleLine(
  resolution: FileResolution,
  hunkIndex: number,
  side: ConflictSide,
  lineIndex: number,
): FileResolution {
  return withHunk(resolution, hunkIndex, (choice) => {
    const values = choice[side];
    if (lineIndex < 0 || lineIndex >= values.length) return choice;
    const next = values.map((value, index) => (index === lineIndex ? !value : value));
    return setSide(choice, side, next);
  });
}

function clearedChoice(choice: HunkChoice, decision: HunkChoice['decision']): HunkChoice {
  return {
    decision,
    ours: choice.ours.map(() => false),
    theirs: choice.theirs.map(() => false),
  };
}

/**
 * "Tomar bloque" / "Quitar bloque". Un lado vacio (una insercion pura, un
 * borrado frente a una modificacion) ya no es un boton muerto: tomarlo deja el
 * bloque resuelto en nada, y volver a pulsarlo lo deshace.
 */
export function toggleBlock(
  file: ConflictFile,
  resolution: FileResolution,
  hunkIndex: number,
  side: ConflictSide,
): FileResolution {
  const hunk = hunksOf(file)[hunkIndex];
  if (hunk === undefined) return resolution;
  return withHunk(resolution, hunkIndex, (choice) => {
    if (hunk[side].length === 0) {
      return clearedChoice(choice, choice.decision === 'empty' ? 'none' : 'empty');
    }
    const takeAll = !choice[side].every(Boolean);
    return setSide(
      choice,
      side,
      choice[side].map(() => takeAll),
    );
  });
}

/** Resolver el bloque a nada de forma explicita. */
export function resolveEmpty(resolution: FileResolution, hunkIndex: number): FileResolution {
  return withHunk(resolution, hunkIndex, (choice) => clearedChoice(choice, 'empty'));
}

/** Volver el bloque a "sin resolver". */
export function clearHunk(resolution: FileResolution, hunkIndex: number): FileResolution {
  return withHunk(resolution, hunkIndex, (choice) => clearedChoice(choice, 'none'));
}

/** "Todo de A" / "Todo de B": ese lado entero en cada bloque, el otro fuera. */
export function takeAll(
  file: ConflictFile,
  resolution: FileResolution,
  side: ConflictSide,
): FileResolution {
  const hunks = hunksOf(file);
  const other: ConflictSide = side === 'ours' ? 'theirs' : 'ours';
  const choices = resolution.hunks.map((choice, index): HunkChoice => {
    const hunk = hunks[index];
    if (hunk === undefined || hunk[side].length === 0) return clearedChoice(choice, 'empty');
    const taken = setSide(
      choice,
      side,
      choice[side].map(() => true),
    );
    return setSide(
      taken,
      other,
      choice[other].map(() => false),
    );
  });
  return { hunks: choices, manual: null, draft: null, acceptMarkers: false };
}
