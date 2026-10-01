import type { PlainKey } from '@ramal/i18n';

import type { ArtifactLabel, DownloadTarget, StoreId } from '../../domain/downloads/release';

/**
 * Las etiquetas visibles de la pagina de descargas. Los datos solo traen claves
 * (ADR 0016: la etiqueta la pone la web, no la base de datos); aqui se cruzan con
 * @ramal/i18n. Al ser Record, una etiqueta nueva sin texto no compila.
 */

export const ARTIFACT_LABELS: Readonly<Record<ArtifactLabel, PlainKey>> = {
  'apple-silicon-dmg': 'web.download.artifact.appleSiliconDmg',
  'intel-dmg': 'web.download.artifact.intelDmg',
  'universal-dmg': 'web.download.artifact.universalDmg',
  'windows-x64-exe': 'web.download.artifact.windowsX64Exe',
  'windows-arm64-exe': 'web.download.artifact.windowsArm64Exe',
  'windows-x64-msi': 'web.download.artifact.windowsX64Msi',
  'linux-deb': 'web.download.artifact.linuxDeb',
  'linux-rpm': 'web.download.artifact.linuxRpm',
  'linux-appimage': 'web.download.artifact.linuxAppImage',
  'linux-arm64-deb': 'web.download.artifact.linuxArm64Deb',
  vsix: 'web.download.artifact.vsix',
};

export const STORE_LABELS: Readonly<Record<StoreId, PlainKey>> = {
  marketplace: 'web.vscode.install',
  'open-vsx': 'web.download.artifact.openVsx',
  flathub: 'web.download.artifact.flathub',
};

export const TARGET_NAMES: Readonly<Record<DownloadTarget, PlainKey>> = {
  mac: 'web.download.os.mac',
  windows: 'web.download.os.windows',
  linux: 'web.download.os.linux',
  vscode: 'web.nav.vscode',
};

export const TARGET_REQUIREMENTS: Readonly<Record<DownloadTarget, PlainKey>> = {
  mac: 'web.download.requirement.mac',
  windows: 'web.download.requirement.windows',
  linux: 'web.download.requirement.linux',
  vscode: 'web.vscode.requirement',
};
