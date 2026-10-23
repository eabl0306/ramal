import { describe, expect, it } from 'vitest';

import { recommendedFile, releaseFiles } from '../../domain/downloads/recommend';
import { VSIX_TARGETS } from '../../domain/downloads/release';

import { createStaticReleaseCatalog } from './static-release-catalog';

describe('createStaticReleaseCatalog', () => {
  it('trae una tarjeta por sistema mas la de VS Code', async () => {
    const release = await createStaticReleaseCatalog().latest();

    expect(release.cards.map((card) => card.target)).toEqual(['mac', 'windows', 'linux', 'vscode']);
    expect(release.cards.map((card) => card.install)).toEqual([
      'brew-cask',
      'winget',
      'flatpak',
      'vscode',
    ]);
  });

  it('cada fichero tiene un id unico y cada sistema tiene su instalador recomendado', async () => {
    const release = await createStaticReleaseCatalog().latest();
    const ids = releaseFiles(release).map((file) => file.id);

    expect(new Set(ids).size).toBe(ids.length);
    // Un Mac nunca dice su procesador en el user agent: el universal funciona seguro.
    expect(recommendedFile(release, { os: 'mac', arch: 'unknown' })?.file).toBe(
      'Git-Rudder-2.4.0-universal.dmg',
    );
    expect(recommendedFile(release, { os: 'mac', arch: 'arm64' })?.file).toBe(
      'Git-Rudder-2.4.0-arm64.dmg',
    );
    expect(recommendedFile(release, { os: 'windows', arch: 'x64' })?.file).toBe(
      'Git-Rudder-Setup-2.4.0-x64.exe',
    );
    expect(recommendedFile(release, { os: 'linux', arch: 'arm64' })?.file).toBe(
      'git_rudder_2.4.0_arm64.deb',
    );
  });

  it('VS Code ofrece un .vsix por cada destino de ADR 0010 y sin tamano inventado', async () => {
    const release = await createStaticReleaseCatalog().latest();
    const vscode = release.cards.find((card) => card.target === 'vscode');
    const vsix = (vscode?.alternates ?? []).flatMap((entry) =>
      entry.kind === 'file' ? [entry.file] : [],
    );

    expect(vsix.map((file) => file.file)).toEqual(
      VSIX_TARGETS.map((target) => `git-rudder-${target}-1.3.0.vsix`),
    );
    expect(vsix.every((file) => file.sizeBytes === null && file.artifact === null)).toBe(true);
  });
});
