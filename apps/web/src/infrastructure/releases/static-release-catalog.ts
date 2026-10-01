import type { ReleaseCatalog } from '../../application/ports/release-catalog';
import type {
  ArtifactLabel,
  DownloadEntry,
  DownloadFile,
  ReleaseInfo,
} from '../../domain/downloads/release';

/**
 * Datos de descarga fijos del primer corte: la version, los ficheros y los
 * tamanos del diseno, con los cambios de gaps X-09 (sin el portable de Windows,
 * el universal de macOS como .dmg). Cuando la API sirva GET /v1/releases/latest
 * (ADR 0016) este adaptador se sustituye por uno HTTP en el composition root.
 */

const MB = 1_000_000;

function file(
  label: ArtifactLabel,
  name: string,
  sizeMb: number | null,
  artifact: DownloadFile['artifact'],
): DownloadEntry {
  const sizeBytes = sizeMb === null ? null : sizeMb * MB;
  return { kind: 'file', file: { id: label, label, file: name, sizeBytes, artifact } };
}

function installer(
  label: ArtifactLabel,
  name: string,
  sizeMb: number | null,
  target: Omit<NonNullable<DownloadFile['artifact']>, 'id'>,
): DownloadEntry {
  return file(label, name, sizeMb, { id: label, ...target });
}

const STATIC_RELEASE: ReleaseInfo = {
  version: '2.4.0',
  releasedAt: '2026-09-30',
  cards: [
    {
      target: 'mac',
      primary: installer('apple-silicon-dmg', 'Ramal-2.4.0-arm64.dmg', 121, {
        os: 'mac',
        arch: 'arm64',
        format: 'dmg',
      }),
      alternates: [
        installer('intel-dmg', 'Ramal-2.4.0-x64.dmg', 118, {
          os: 'mac',
          arch: 'x64',
          format: 'dmg',
        }),
        installer('universal-dmg', 'Ramal-2.4.0-universal.dmg', 231, {
          os: 'mac',
          arch: 'universal',
          format: 'dmg',
        }),
      ],
      install: 'brew-cask',
    },
    {
      target: 'windows',
      primary: installer('windows-x64-exe', 'Ramal-Setup-2.4.0-x64.exe', 102, {
        os: 'windows',
        arch: 'x64',
        format: 'exe',
      }),
      alternates: [
        installer('windows-arm64-exe', 'Ramal-Setup-2.4.0-arm64.exe', 104, {
          os: 'windows',
          arch: 'arm64',
          format: 'exe',
        }),
        installer('windows-x64-msi', 'Ramal-2.4.0-x64.msi', 109, {
          os: 'windows',
          arch: 'x64',
          format: 'msi',
        }),
      ],
      install: 'winget',
    },
    {
      target: 'linux',
      primary: installer('linux-deb', 'ramal_2.4.0_amd64.deb', 92, {
        os: 'linux',
        arch: 'x64',
        format: 'deb',
      }),
      alternates: [
        installer('linux-rpm', 'ramal-2.4.0.x86_64.rpm', 97, {
          os: 'linux',
          arch: 'x64',
          format: 'rpm',
        }),
        installer('linux-appimage', 'Ramal-2.4.0.AppImage', 112, {
          os: 'linux',
          arch: 'x64',
          format: 'appimage',
        }),
        { kind: 'store', store: 'flathub' },
        installer('linux-arm64-deb', 'ramal_2.4.0_arm64.deb', 94, {
          os: 'linux',
          arch: 'arm64',
          format: 'deb',
        }),
      ],
      install: 'flatpak',
    },
    {
      target: 'vscode',
      primary: { kind: 'store', store: 'marketplace' },
      alternates: [{ kind: 'store', store: 'open-vsx' }, file('vsix', 'ramal-1.3.0.vsix', 6, null)],
      install: 'vscode',
    },
  ],
};

export function createStaticReleaseCatalog(): ReleaseCatalog {
  return { latest: () => Promise.resolve(STATIC_RELEASE) };
}
