import { buildOutput } from './build-output';
import type { ConflictFile, ConflictLabels, FileResolution } from './model';

/**
 * Edicion a mano del resultado. El borrador vive en la resolucion de cada
 * fichero, asi que cambiar de fichero no lo pierde (gaps B-22), y guardarlo no
 * toca las elecciones: "Restablecer" vuelve a ellas.
 */

export function isEditingManually(resolution: FileResolution): boolean {
  return resolution.draft !== null;
}

/**
 * Abre el editor. Si ya habia borrador se retoma; si no, parte del texto guardado
 * o del resultado actual, con marcadores en los bloques sin resolver.
 */
export function startManualEdit(
  file: ConflictFile,
  resolution: FileResolution,
  labels: ConflictLabels,
): FileResolution {
  if (resolution.draft !== null) return resolution;
  const draft = resolution.manual ?? buildOutput(file, resolution, labels).text;
  return { ...resolution, draft };
}

export function updateManualDraft(resolution: FileResolution, text: string): FileResolution {
  return { ...resolution, draft: text };
}

/** "Listo": el borrador pasa a ser el texto manual. */
export function saveManualEdit(resolution: FileResolution): FileResolution {
  if (resolution.draft === null) return resolution;
  return { ...resolution, manual: resolution.draft, draft: null, acceptMarkers: false };
}

/** Cierra el editor sin guardar el borrador. */
export function cancelManualEdit(resolution: FileResolution): FileResolution {
  return { ...resolution, draft: null };
}

/** "Restablecer": fuera el texto manual, vuelven a mandar las elecciones. */
export function resetManualEdit(resolution: FileResolution): FileResolution {
  return { ...resolution, manual: null, draft: null, acceptMarkers: false };
}

/** "Marcar como resuelto igualmente". Solo tiene efecto sobre un texto manual guardado. */
export function acceptMarkersAnyway(resolution: FileResolution): FileResolution {
  return resolution.manual === null ? resolution : { ...resolution, acceptMarkers: true };
}
