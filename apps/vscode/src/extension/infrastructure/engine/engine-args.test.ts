import { describe, expect, it } from 'vitest';

import { engineArgs } from './engine-args';

describe('argumentos del sidecar', () => {
  it('pasa el git de VS Code con --git-path antes del nivel de log', () => {
    expect(engineArgs({ gitPath: '/opt/git/bin/git', logLevel: 'debug' })).toEqual([
      '--git-path',
      '/opt/git/bin/git',
      '--log-level',
      'debug',
    ]);
  });

  it('sin git de VS Code no pasa --git-path y el sidecar busca en el PATH', () => {
    expect(engineArgs({ gitPath: undefined, logLevel: 'info' })).toEqual(['--log-level', 'info']);
  });

  it('una ruta vacia no llega como --git-path', () => {
    expect(engineArgs({ gitPath: '', logLevel: 'info' })).toEqual(['--log-level', 'info']);
  });

  it('una ruta con espacios o signos raros va entera en su propio argumento', () => {
    const gitPath = 'C:\\Program Files\\Git=portable\\cmd\\git.exe';
    expect(engineArgs({ gitPath, logLevel: 'info' }).slice(0, 2)).toEqual(['--git-path', gitPath]);
  });
});
