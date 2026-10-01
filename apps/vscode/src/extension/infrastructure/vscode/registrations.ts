import * as vscode from 'vscode';

import { describeError } from '../../application/engine-errors';
import type { Logger } from '../../application/ports/logger';

/** Ids que el manifiesto declara en `contributes`. */
export const COMMAND_OPEN_GRAPH = 'ramal.openGraph';
const WELCOME_VIEW = 'ramal.welcome';

export interface CommandHandlers {
  readonly openGraph: () => Promise<unknown>;
}

/**
 * Registra los comandos de la paleta. Un fallo inesperado se registra en el log
 * en vez de perderse como promesa rechazada: los esperados ya los avisa el
 * caso de uso.
 */
export function registerCommands(handlers: CommandHandlers, logger: Logger): vscode.Disposable[] {
  return [
    vscode.commands.registerCommand(COMMAND_OPEN_GRAPH, async () => {
      try {
        await handlers.openGraph();
      } catch (error) {
        logger.error(`${COMMAND_OPEN_GRAPH} failed: ${describeError(error)}`);
      }
    }),
  ];
}

/**
 * La vista de la barra de actividad. En el primer corte no tiene elementos:
 * VS Code ensena entonces su contenido de bienvenida (`viewsWelcome`), con el
 * boton de abrir el grafo. Las vistas de ramas, remotos y stashes llegaran aqui.
 */
export function registerWelcomeView(): vscode.Disposable {
  const provider: vscode.TreeDataProvider<never> = {
    getTreeItem: (element) => element,
    getChildren: () => [],
  };
  return vscode.window.registerTreeDataProvider(WELCOME_VIEW, provider);
}
