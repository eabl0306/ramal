import type * as vscode from 'vscode';

import type { HostProbe } from '../application/use-cases/host-probe';

import { composeExtension } from './composition-root';

/**
 * Entrada de la extension (`main` del manifiesto, empaquetada por esbuild en
 * `dist/extension.cjs`). Activar no arranca el motor: lo arranca la primera
 * peticion. Si espera al git de VS Code (API de `vscode.git`), que el motor
 * necesita al lanzarse. La desactivacion la hace VS Code liberando
 * `context.subscriptions`.
 */
export function activate(context: vscode.ExtensionContext): Promise<HostProbe | undefined> {
  return composeExtension(context);
}
