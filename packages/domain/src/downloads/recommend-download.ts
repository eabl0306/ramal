import {
  type CpuArch,
  type DesktopOs,
  type DetectedPlatform,
  isDesktopOs,
} from './detect-platform';

/**
 * Que descarga destacar. Los artefactos los da quien publica (hoy datos fijos,
 * mas adelante el manifiesto de la API); esto solo decide cual encaja mejor con
 * el equipo de quien visita la pagina.
 */

export interface DownloadArtifact {
  readonly id: string;
  readonly os: DesktopOs;
  readonly arch: CpuArch | 'universal';
  /** Extension sin punto: dmg, zip, exe, msi, deb, rpm, appimage... */
  readonly format: string;
}

export interface DownloadRecommendation {
  readonly os: DesktopOs;
  readonly artifact: DownloadArtifact;
}

/** El instalador de doble clic primero; los formatos que no estan aqui van al final. */
const PREFERRED_FORMATS: Readonly<Record<DesktopOs, readonly string[]>> = {
  mac: ['dmg', 'zip'],
  windows: ['exe', 'msi', 'zip'],
  linux: ['deb', 'rpm', 'appimage'],
};

/** La arquitectura mas comun de cada sistema, para cuando no se sabe la del visitante. */
const DEFAULT_ARCH: Readonly<Record<DesktopOs, Exclude<CpuArch, 'unknown'>>> = {
  mac: 'arm64',
  windows: 'x64',
  linux: 'x64',
};

/** Penalizacion por arquitectura: 0 nativo, 2 solo con emulacion (Rosetta, Prism). */
function archPenalty(artifact: DownloadArtifact, arch: CpuArch, os: DesktopOs): number {
  if (artifact.arch === 'universal') return 0;
  const wanted = arch === 'unknown' ? DEFAULT_ARCH[os] : arch;
  return artifact.arch === wanted ? 0 : 2;
}

function formatRank(artifact: DownloadArtifact, os: DesktopOs): number {
  const formats = PREFERRED_FORMATS[os];
  const index = formats.indexOf(artifact.format.toLowerCase());
  return index === -1 ? formats.length : index;
}

/**
 * Desempate entre un binario nativo y uno universal del mismo formato: si se
 * conoce la arquitectura, el nativo (pesa menos); si no, el universal (funciona
 * seguro).
 */
function universalTiebreak(artifact: DownloadArtifact, arch: CpuArch): number {
  const isUniversal = artifact.arch === 'universal';
  if (arch === 'unknown') return isUniversal ? 0 : 1;
  return isUniversal ? 1 : 0;
}

function score(artifact: DownloadArtifact, platform: DetectedPlatform, os: DesktopOs): number[] {
  return [
    archPenalty(artifact, platform.arch, os),
    formatRank(artifact, os),
    universalTiebreak(artifact, platform.arch),
  ];
}

function compareScores(a: readonly number[], b: readonly number[]): number {
  for (const [index, value] of a.entries()) {
    const other = b[index] ?? 0;
    if (value !== other) return value - other;
  }
  return 0;
}

/**
 * El artefacto recomendado, o null si no hay nada que destacar: en un movil o en
 * un sistema desconocido la pagina ensena todas las tarjetas sin marcar ninguna.
 */
export function recommendDownload(
  platform: DetectedPlatform,
  artifacts: readonly DownloadArtifact[],
): DownloadRecommendation | null {
  const { os } = platform;
  if (!isDesktopOs(os)) return null;
  const ranked = artifacts
    .filter((artifact) => artifact.os === os)
    .map((artifact) => ({ artifact, score: score(artifact, platform, os) }))
    .sort((a, b) => compareScores(a.score, b.score));
  const best = ranked[0];
  return best === undefined ? null : { os, artifact: best.artifact };
}

/** La tarjeta que lleva "Tu sistema", o null si no se debe destacar ninguna. */
export function highlightedDownloadOs(platform: DetectedPlatform): DesktopOs | null {
  return isDesktopOs(platform.os) ? platform.os : null;
}
