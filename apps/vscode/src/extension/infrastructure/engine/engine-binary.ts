import { existsSync } from 'node:fs';
import path from 'node:path';

/** Como se ejecuta la extension; refleja `vscode.ExtensionMode`. */
export type HostMode = 'production' | 'development' | 'test';

export interface EngineBinaryInput {
  /** Carpeta de la extension (`context.extensionPath`). */
  readonly extensionPath: string;
  readonly mode: HostMode;
  readonly platform: string;
  /** `gitRudder.engine.path` a nivel de usuario; solo cuenta fuera de produccion. */
  readonly configuredPath?: string | undefined;
  /** `GIT_RUDDER_ENGINE_PATH`; solo cuenta fuera de produccion. */
  readonly environmentPath?: string | undefined;
  /** Para los tests; por defecto mira el disco. */
  readonly exists?: (file: string) => boolean;
}

/** Nombre del binario del sidecar en esta plataforma. */
export function engineBinaryName(platform: string): string {
  return platform === 'win32' ? 'git-rudder-vscode-engine.exe' : 'git-rudder-vscode-engine';
}

/**
 * Que binario se ejecuta (architecture.md 4.4). En produccion siempre el de la
 * VSIX, `dist/bin/git-rudder-vscode-engine`: un ajuste que eligiera el binario seria
 * ejecucion de codigo con solo abrir una carpeta. En desarrollo y en los tests
 * de integracion se acepta el ajuste de usuario, la variable de entorno o lo
 * que haya compilado cargo en `target/` del monorepo, por ese orden.
 */
export function resolveEngineBinary(input: EngineBinaryInput): string {
  const name = engineBinaryName(input.platform);
  const packaged = path.join(input.extensionPath, 'dist', 'bin', name);
  if (input.mode === 'production') return packaged;
  const explicit = [input.configuredPath, input.environmentPath].find(
    (candidate): candidate is string => candidate !== undefined && candidate.trim() !== '',
  );
  if (explicit !== undefined) return explicit;
  const exists = input.exists ?? existsSync;
  const target = path.join(input.extensionPath, '..', '..', 'target');
  const built = ['debug', 'release']
    .map((profile) => path.join(target, profile, name))
    .find((candidate) => exists(candidate));
  return built ?? packaged;
}
