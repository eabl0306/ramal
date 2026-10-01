import { describe, expect, it } from 'vitest';

import { detectOs, detectPlatform, isDesktopOs } from './detect-platform';
import {
  type DownloadArtifact,
  highlightedDownloadOs,
  recommendDownload,
} from './recommend-download';

const UA = {
  mac: 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 Version/18.0 Safari/605.1.15',
  windows:
    'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 Chrome/141.0 Safari/537.36',
  windowsArm: 'Mozilla/5.0 (Windows NT 10.0; ARM64) AppleWebKit/537.36 Chrome/141.0 Safari/537.36',
  linux: 'Mozilla/5.0 (X11; Linux x86_64; rv:143.0) Gecko/20100101 Firefox/143.0',
  linuxArm: 'Mozilla/5.0 (X11; Linux aarch64; rv:143.0) Gecko/20100101 Firefox/143.0',
  android:
    'Mozilla/5.0 (Linux; Android 15; Pixel 9) AppleWebKit/537.36 Chrome/141.0 Mobile Safari/537.36',
  iphone:
    'Mozilla/5.0 (iPhone; CPU iPhone OS 18_0 like Mac OS X) AppleWebKit/605.1.15 Mobile/15E148',
  chromebook: 'Mozilla/5.0 (X11; CrOS x86_64 16093.0.0) AppleWebKit/537.36 Chrome/141.0',
  unknown: 'Mozilla/5.0 (PlayStation 5) AppleWebKit/605.1.15',
};

/** Los artefactos de la pagina de descargas del prototipo. */
const ARTIFACTS: readonly DownloadArtifact[] = [
  { id: 'mac-arm', os: 'mac', arch: 'arm64', format: 'dmg' },
  { id: 'mac-intel', os: 'mac', arch: 'x64', format: 'dmg' },
  { id: 'mac-universal', os: 'mac', arch: 'universal', format: 'zip' },
  { id: 'win-x64', os: 'windows', arch: 'x64', format: 'exe' },
  { id: 'win-arm', os: 'windows', arch: 'arm64', format: 'exe' },
  { id: 'win-msi', os: 'windows', arch: 'x64', format: 'msi' },
  { id: 'deb', os: 'linux', arch: 'x64', format: 'deb' },
  { id: 'rpm', os: 'linux', arch: 'x64', format: 'rpm' },
  { id: 'deb-arm', os: 'linux', arch: 'arm64', format: 'deb' },
  { id: 'flatpak', os: 'linux', arch: 'x64', format: 'flatpak' },
];

function recommended(userAgent: string, extra: object = {}): string | null {
  const platform = detectPlatform({ userAgent, ...extra });
  return recommendDownload(platform, ARTIFACTS)?.artifact.id ?? null;
}

describe('detectOs', () => {
  it('reconoce los sistemas de escritorio', () => {
    expect(detectOs({ userAgent: UA.mac })).toBe('mac');
    expect(detectOs({ userAgent: UA.windows })).toBe('windows');
    expect(detectOs({ userAgent: UA.linux })).toBe('linux');
  });

  it('Android e iOS no pasan por macOS ni por Linux', () => {
    expect(detectOs({ userAgent: UA.android })).toBe('android');
    expect(detectOs({ userAgent: UA.iphone })).toBe('ios');
    expect(detectOs({ userAgent: UA.mac, maxTouchPoints: 5 })).toBe('ios');
  });

  it('lo desconocido y ChromeOS no se adivinan', () => {
    expect(detectOs({ userAgent: UA.unknown })).toBe('unknown');
    expect(detectOs({ userAgent: UA.chromebook })).toBe('unknown');
  });

  it('la plataforma de userAgentData manda sobre el user agent', () => {
    expect(detectOs({ userAgent: UA.windows, uaPlatform: 'macOS' })).toBe('mac');
    expect(detectOs({ userAgent: UA.linux, uaPlatform: 'Desconocida' })).toBe('linux');
  });
});

describe('detectPlatform', () => {
  it('saca la arquitectura del user agent fuera de macOS', () => {
    expect(detectPlatform({ userAgent: UA.windows }).arch).toBe('x64');
    expect(detectPlatform({ userAgent: UA.windowsArm }).arch).toBe('arm64');
    expect(detectPlatform({ userAgent: UA.linuxArm }).arch).toBe('arm64');
    expect(detectPlatform({ userAgent: UA.unknown }).arch).toBe('unknown');
  });

  it('en macOS el user agent miente y solo valen las pistas', () => {
    expect(detectPlatform({ userAgent: UA.mac }).arch).toBe('unknown');
    expect(detectPlatform({ userAgent: UA.mac, architecture: 'arm' }).arch).toBe('arm64');
    expect(detectPlatform({ userAgent: UA.mac, architecture: 'x86', bitness: '64' }).arch).toBe(
      'x64',
    );
  });

  it('ignora pistas de 32 bits o desconocidas', () => {
    expect(detectPlatform({ userAgent: UA.mac, architecture: 'x86', bitness: '32' }).arch).toBe(
      'unknown',
    );
    expect(detectPlatform({ userAgent: UA.mac, architecture: 'riscv' }).arch).toBe('unknown');
  });
});

describe('recommendDownload', () => {
  it('elige el instalador nativo de cada sistema', () => {
    expect(recommended(UA.windows)).toBe('win-x64');
    expect(recommended(UA.windowsArm)).toBe('win-arm');
    expect(recommended(UA.linux)).toBe('deb');
    expect(recommended(UA.linuxArm)).toBe('deb-arm');
  });

  it('en un Mac sin pistas ofrece el dmg de Apple Silicon, como el prototipo', () => {
    expect(recommended(UA.mac)).toBe('mac-arm');
    expect(recommended(UA.mac, { architecture: 'x86', bitness: '64' })).toBe('mac-intel');
  });

  it('prefiere el universal si no se sabe la arquitectura y empata el formato', () => {
    const artifacts: DownloadArtifact[] = [
      { id: 'arm', os: 'mac', arch: 'arm64', format: 'dmg' },
      { id: 'uni', os: 'mac', arch: 'universal', format: 'dmg' },
    ];
    const pick = (arch: 'unknown' | 'arm64'): string | undefined =>
      recommendDownload({ os: 'mac', arch }, artifacts)?.artifact.id;
    expect(pick('unknown')).toBe('uni');
    expect(pick('arm64')).toBe('arm');
  });

  it('cae a otra arquitectura si no hay nativa', () => {
    const only = [{ id: 'x64', os: 'windows', arch: 'x64', format: 'exe' }] as const;
    expect(recommendDownload({ os: 'windows', arch: 'arm64' }, only)?.artifact.id).toBe('x64');
  });

  it('no recomienda nada en un movil, en un sistema desconocido o sin artefactos', () => {
    expect(recommended(UA.android)).toBeNull();
    expect(recommended(UA.iphone)).toBeNull();
    expect(recommended(UA.unknown)).toBeNull();
    expect(recommendDownload({ os: 'linux', arch: 'x64' }, [])).toBeNull();
  });
});

describe('highlightedDownloadOs', () => {
  it('solo destaca tarjetas de escritorio', () => {
    expect(highlightedDownloadOs({ os: 'linux', arch: 'x64' })).toBe('linux');
    expect(highlightedDownloadOs({ os: 'ios', arch: 'unknown' })).toBeNull();
    expect(isDesktopOs('android')).toBe(false);
  });
});
