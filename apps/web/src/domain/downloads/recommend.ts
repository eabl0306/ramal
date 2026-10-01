import { type DetectedPlatform, recommendDownload } from '@ramal/domain';

import type { DownloadEntry, DownloadFile, ReleaseInfo } from './release';

/**
 * Que ensena la landing segun el equipo de quien la visita. La regla de que
 * instalador encaja vive en @ramal/domain; esto solo la aplica a los datos de
 * una publicacion.
 */

function fileOf(entry: DownloadEntry): DownloadFile[] {
  return entry.kind === 'file' ? [entry.file] : [];
}

/** Todos los ficheros de todas las tarjetas, principales y alternativos. */
export function releaseFiles(release: ReleaseInfo): DownloadFile[] {
  return release.cards.flatMap((card) => [card.primary, ...card.alternates].flatMap(fileOf));
}

/** El instalador para el boton principal de la portada, o null en un movil o un sistema raro. */
export function recommendedFile(
  release: ReleaseInfo,
  platform: DetectedPlatform,
): DownloadFile | null {
  const files = releaseFiles(release);
  const artifacts = files.flatMap((file) => (file.artifact === null ? [] : [file.artifact]));
  const choice = recommendDownload(platform, artifacts);
  if (choice === null) return null;
  return files.find((file) => file.id === choice.artifact.id) ?? null;
}

/** Un movil no puede instalar Ramal: la pagina lo dice en vez de destacar macOS (gaps B-55). */
export function isMobileVisitor(platform: DetectedPlatform): boolean {
  return platform.os === 'android' || platform.os === 'ios';
}
