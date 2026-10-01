import * as vscode from 'vscode';

import { describeError } from '../../application/engine-errors';
import type { Logger } from '../../application/ports/logger';

/** Lo poco que se usa de la API de la extension `vscode.git`, tipado aqui. */
interface GitExtensionExports {
  readonly enabled: boolean;
  getAPI(version: 1): { readonly git: { readonly path: string } };
}

/** Lo que se espera a `vscode.git` antes de seguir con el git del `PATH`. */
const GIT_EXTENSION_TIMEOUT_MS = 10_000;

/**
 * El git que usa VS Code: el `git.path` que resolvio la extension `vscode.git`
 * (ADR 0009, punto 1). Existe por un fallo real: el host no lo pasaba y quien
 * tiene git solo en `git.path` (Git portable en Windows, un Homebrew detras del
 * shim de Xcode) veia otro git o ninguno.
 *
 * Devuelve `undefined` si la extension falta, esta desactivada, no encontro git
 * o tarda demasiado: el sidecar usa entonces el primer git del `PATH`. No se
 * declara `extensionDependencies` sobre `vscode.git` porque impediria activar
 * Ramal a quien la tenga desactivada.
 */
export async function resolveGitPath(logger: Logger): Promise<string | undefined> {
  const extension = vscode.extensions.getExtension<GitExtensionExports>('vscode.git');
  if (extension === undefined) return undefined;
  let timer: ReturnType<typeof setTimeout> | undefined;
  const timeout = new Promise<undefined>((resolve) => {
    timer = setTimeout(resolve, GIT_EXTENSION_TIMEOUT_MS, undefined);
  });
  try {
    return await Promise.race([gitPathOf(extension), timeout]);
  } catch (error) {
    logger.warn(`vscode.git gave no git path: ${describeError(error)}`);
    return undefined;
  } finally {
    clearTimeout(timer);
  }
}

async function gitPathOf(
  extension: vscode.Extension<GitExtensionExports>,
): Promise<string | undefined> {
  const git = extension.isActive ? extension.exports : await extension.activate();
  return git.enabled ? git.getAPI(1).git.path : undefined;
}
