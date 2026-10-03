import type { DesktopOs, DownloadArtifact } from '@git-rudder/domain';

/**
 * La pagina de descargas: una tarjeta por sistema mas la de VS Code, cada una con
 * su descarga principal, sus alternativas y su comando de instalacion. Las
 * etiquetas visibles no viajan con los datos: son claves que traduce la interfaz
 * (ADR 0016), igual que el comando sale de una tabla fija y no de un texto libre.
 */

/**
 * Destinos de `vsce package --target` (ADR 0010). La extension lleva el binario
 * del motor, asi que hay un .vsix por plataforma y no uno universal (gaps X-09).
 */
export const VSIX_TARGETS = [
  'win32-x64',
  'win32-arm64',
  'darwin-x64',
  'darwin-arm64',
  'linux-x64',
  'linux-arm64',
  'alpine-x64',
  'alpine-arm64',
] as const;

export type VsixTarget = (typeof VSIX_TARGETS)[number];

export type ArtifactLabel =
  | 'apple-silicon-dmg'
  | 'intel-dmg'
  | 'universal-dmg'
  | 'windows-x64-exe'
  | 'windows-arm64-exe'
  | 'windows-x64-msi'
  | 'linux-deb'
  | 'linux-rpm'
  | 'linux-appimage'
  | 'linux-arm64-deb'
  | `vsix-${VsixTarget}`;

export type StoreId = 'marketplace' | 'open-vsx' | 'flathub';

export type InstallMethod = 'brew-cask' | 'winget' | 'flatpak' | 'vscode';

export type DownloadTarget = DesktopOs | 'vscode';

export interface DownloadFile {
  readonly id: string;
  readonly label: ArtifactLabel;
  /** Nombre del fichero publicado. */
  readonly file: string;
  /** null cuando no se conoce: la tarjeta no ensena tamano. */
  readonly sizeBytes: number | null;
  /** Sistema, arquitectura y formato de un instalador de escritorio; null en los .vsix. */
  readonly artifact: DownloadArtifact | null;
}

export type DownloadEntry =
  | { readonly kind: 'file'; readonly file: DownloadFile }
  | { readonly kind: 'store'; readonly store: StoreId };

export interface DownloadCard {
  readonly target: DownloadTarget;
  readonly primary: DownloadEntry;
  readonly alternates: readonly DownloadEntry[];
  readonly install: InstallMethod;
}

export interface ReleaseInfo {
  readonly version: string;
  /** Fecha de publicacion, YYYY-MM-DD. */
  readonly releasedAt: string;
  readonly cards: readonly DownloadCard[];
}

/** Los comandos que la landing invita a pegar en una terminal: nunca texto libre (ADR 0016). */
export const INSTALL_COMMANDS: Readonly<Record<InstallMethod, string>> = {
  'brew-cask': 'brew install --cask git-rudder',
  winget: 'winget install Git Rudder.Git Rudder',
  flatpak: 'flatpak install flathub com.git_rudder.GitRudder',
  vscode: 'code --install-extension git-rudder.git-rudder',
};

/** Las tiendas son URL fijas del codigo, no datos de una publicacion (ADR 0016). */
export const STORE_URLS: Readonly<Record<StoreId, string>> = {
  marketplace: 'https://marketplace.visualstudio.com/items?itemName=git-rudder.git-rudder',
  'open-vsx': 'https://open-vsx.org/extension/git-rudder/git-rudder',
  flathub: 'https://flathub.org/apps/com.git_rudder.GitRudder',
};

/** Megabytes decimales, como los ensena el diseno ("118 MB"). */
export function sizeInMegabytes(bytes: number): number {
  return Math.max(1, Math.round(bytes / 1_000_000));
}

/** La pildora de la portada lleva la version corta: "2.4.0" es "v2.4". */
export function shortVersion(version: string): string {
  const [major = version, minor] = version.split('.');
  return minor === undefined ? `v${major}` : `v${major}.${minor}`;
}

/** El dia de publicacion a medianoche UTC, para formatearlo sin saltar de dia por la zona. */
export function releaseDay(releasedAt: string): Date {
  return new Date(`${releasedAt}T00:00:00Z`);
}
