import { type ConflictFile, type FileResolution, initResolution } from './model';
import { type MergeProgress, mergeProgress } from './resolved';
import type { ConflictOperation } from './side-roles';

/**
 * El estado completo del resolvedor para una operacion en curso: los ficheros
 * en conflicto, la resolucion de cada uno y cual se esta viendo. Es lo unico que
 * la interfaz guarda; el resto se deriva con las funciones del modulo.
 */
export interface ConflictSession {
  readonly operation: ConflictOperation;
  readonly files: readonly ConflictFile[];
  readonly resolutions: readonly FileResolution[];
  readonly activeFile: number;
}

export function createConflictSession(
  operation: ConflictOperation,
  files: readonly ConflictFile[],
): ConflictSession {
  return { operation, files, resolutions: files.map(initResolution), activeFile: 0 };
}

/** Cambia de fichero. Los borradores se quedan en su fichero, no se pierden. */
export function selectConflictFile(session: ConflictSession, index: number): ConflictSession {
  if (index < 0 || index >= session.files.length) return session;
  return { ...session, activeFile: index };
}

export function updateConflictResolution(
  session: ConflictSession,
  index: number,
  update: (resolution: FileResolution) => FileResolution,
): ConflictSession {
  const current = session.resolutions[index];
  if (current === undefined) return session;
  const resolutions = session.resolutions.map((resolution, position) =>
    position === index ? update(current) : resolution,
  );
  return { ...session, resolutions };
}

export function conflictSessionProgress(session: ConflictSession): MergeProgress {
  return mergeProgress(session.resolutions);
}
