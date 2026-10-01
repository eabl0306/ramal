import { hasConflictMarkers } from './markers';
import type { FileResolution, HunkChoice } from './model';

/**
 * La regla de resuelto. Con edicion manual, el texto no puede tener marcadores
 * (salvo que quien edita diga que son legitimos); sin ella, cada bloque necesita
 * una decision: lineas elegidas o vacio explicito. Un fichero sin bloques esta
 * resuelto.
 */

export function isHunkResolved(choice: HunkChoice): boolean {
  return choice.decision !== 'none';
}

export function hasManualEdit(resolution: FileResolution): boolean {
  return resolution.manual !== null || resolution.draft !== null;
}

export function isFileResolved(resolution: FileResolution): boolean {
  if (resolution.manual !== null) {
    return resolution.acceptMarkers || !hasConflictMarkers(resolution.manual);
  }
  return resolution.hunks.every(isHunkResolved);
}

export interface MergeProgress {
  readonly resolved: number;
  readonly total: number;
  /** Solo con todo resuelto se puede completar el merge. */
  readonly complete: boolean;
}

export function mergeProgress(resolutions: readonly FileResolution[]): MergeProgress {
  const resolved = resolutions.filter(isFileResolved).length;
  return { resolved, total: resolutions.length, complete: resolved === resolutions.length };
}
