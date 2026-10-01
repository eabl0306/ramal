import { describe, expect, it } from 'vitest';

import { isMobileVisitor, recommendedFile, releaseFiles } from './recommend';
import {
  type DownloadFile,
  type ReleaseInfo,
  releaseDay,
  shortVersion,
  sizeInMegabytes,
} from './release';

function installer(id: string, artifact: DownloadFile['artifact']): DownloadFile {
  return { id, label: 'linux-deb', file: `${id}.bin`, sizeBytes: null, artifact };
}

const MAC_ARM = installer('mac-arm', { id: 'mac-arm', os: 'mac', arch: 'arm64', format: 'dmg' });
const MAC_X64 = installer('mac-x64', { id: 'mac-x64', os: 'mac', arch: 'x64', format: 'dmg' });
const WIN_X64 = installer('win-x64', { id: 'win-x64', os: 'windows', arch: 'x64', format: 'exe' });
const VSIX = installer('vsix', null);

const RELEASE: ReleaseInfo = {
  version: '2.4.0',
  releasedAt: '2026-09-30',
  cards: [
    {
      target: 'mac',
      primary: { kind: 'file', file: MAC_ARM },
      alternates: [{ kind: 'file', file: MAC_X64 }],
      install: 'brew-cask',
    },
    {
      target: 'windows',
      primary: { kind: 'file', file: WIN_X64 },
      alternates: [],
      install: 'winget',
    },
    {
      target: 'vscode',
      primary: { kind: 'store', store: 'marketplace' },
      alternates: [{ kind: 'file', file: VSIX }],
      install: 'vscode',
    },
  ],
};

describe('releaseFiles', () => {
  it('junta los ficheros principales y alternativos y deja fuera las tiendas', () => {
    expect(releaseFiles(RELEASE).map((file) => file.id)).toEqual([
      'mac-arm',
      'mac-x64',
      'win-x64',
      'vsix',
    ]);
  });
});

describe('recommendedFile', () => {
  it('elige el instalador nativo del sistema y la arquitectura del visitante', () => {
    expect(recommendedFile(RELEASE, { os: 'mac', arch: 'x64' })?.id).toBe('mac-x64');
    expect(recommendedFile(RELEASE, { os: 'windows', arch: 'unknown' })?.id).toBe('win-x64');
  });

  it('en un Mac sin pistas de arquitectura recomienda Apple Silicon', () => {
    expect(recommendedFile(RELEASE, { os: 'mac', arch: 'unknown' })?.id).toBe('mac-arm');
  });

  it('no recomienda nada en un movil ni si el sistema no tiene instalador', () => {
    expect(recommendedFile(RELEASE, { os: 'ios', arch: 'unknown' })).toBeNull();
    expect(recommendedFile(RELEASE, { os: 'linux', arch: 'x64' })).toBeNull();
  });
});

describe('isMobileVisitor', () => {
  it('reconoce Android e iOS y nada mas', () => {
    expect(isMobileVisitor({ os: 'android', arch: 'arm64' })).toBe(true);
    expect(isMobileVisitor({ os: 'ios', arch: 'unknown' })).toBe(true);
    expect(isMobileVisitor({ os: 'linux', arch: 'x64' })).toBe(false);
    expect(isMobileVisitor({ os: 'unknown', arch: 'unknown' })).toBe(false);
  });
});

describe('shortVersion y releaseDay', () => {
  it('acorta la version a mayor y menor', () => {
    expect(shortVersion('2.4.0')).toBe('v2.4');
    expect(shortVersion('3')).toBe('v3');
  });

  it('lee el dia de publicacion en UTC', () => {
    expect(releaseDay('2026-09-30').toISOString()).toBe('2026-09-30T00:00:00.000Z');
  });
});

describe('sizeInMegabytes', () => {
  it('redondea a megabytes decimales y nunca ensena cero', () => {
    expect(sizeInMegabytes(118_400_000)).toBe(118);
    expect(sizeInMegabytes(10_000)).toBe(1);
  });
});
