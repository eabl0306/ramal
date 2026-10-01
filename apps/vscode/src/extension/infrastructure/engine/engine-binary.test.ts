import path from 'node:path';

import { describe, expect, it } from 'vitest';

import { type EngineBinaryInput, engineBinaryName, resolveEngineBinary } from './engine-binary';

const EXTENSION = path.join('/repo', 'apps', 'vscode');
const PACKAGED = path.join(EXTENSION, 'dist', 'bin', 'ramal-vscode-engine');
const DEBUG = path.join('/repo', 'target', 'debug', 'ramal-vscode-engine');
const RELEASE = path.join('/repo', 'target', 'release', 'ramal-vscode-engine');

function input(overrides: Partial<EngineBinaryInput>): EngineBinaryInput {
  return {
    extensionPath: EXTENSION,
    mode: 'development',
    platform: 'linux',
    exists: () => false,
    ...overrides,
  };
}

describe('binario del motor', () => {
  it('en produccion siempre es el de la VSIX, diga lo que diga la configuracion', () => {
    const resolved = resolveEngineBinary(
      input({
        mode: 'production',
        configuredPath: '/tmp/evil',
        environmentPath: '/tmp/evil2',
        exists: () => true,
      }),
    );
    expect(resolved).toBe(PACKAGED);
  });

  it('en desarrollo manda el ajuste de usuario y despues la variable de entorno', () => {
    expect(
      resolveEngineBinary(input({ configuredPath: '/opt/engine', environmentPath: '/env' })),
    ).toBe('/opt/engine');
    expect(resolveEngineBinary(input({ configuredPath: '  ', environmentPath: '/env' }))).toBe(
      '/env',
    );
  });

  it('sin ajustes usa lo que compilo cargo, debug antes que release', () => {
    expect(resolveEngineBinary(input({ exists: () => true }))).toBe(DEBUG);
    expect(resolveEngineBinary(input({ mode: 'test', exists: (file) => file === RELEASE }))).toBe(
      RELEASE,
    );
  });

  it('si no hay nada compilado vuelve al de la VSIX', () => {
    expect(resolveEngineBinary(input({}))).toBe(PACKAGED);
  });

  it('en Windows el binario lleva .exe', () => {
    expect(engineBinaryName('win32')).toBe('ramal-vscode-engine.exe');
    expect(engineBinaryName('darwin')).toBe('ramal-vscode-engine');
  });
});
