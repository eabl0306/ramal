import * as vscode from 'vscode';

import type { Dialogs, PickItem } from '../../application/ports/dialogs';
import type { Notifications } from '../../application/ports/notifications';
import type { WorkspaceRepositories } from '../../application/ports/workspace-repositories';

/** Avisos con `window.show*Message`. */
export const vscodeNotifications: Notifications = {
  showInfo(message) {
    void vscode.window.showInformationMessage(message);
  },
  async showError(message, ...actions) {
    return vscode.window.showErrorMessage(message, ...actions);
  },
};

interface QuickPickEntry<T> extends vscode.QuickPickItem {
  readonly value: T;
}

/** Listas con `window.showQuickPick`. */
export const vscodeDialogs: Dialogs = {
  async pick<T>(items: readonly PickItem<T>[], placeholder: string): Promise<T | undefined> {
    const entries: QuickPickEntry<T>[] = items.map((item) => ({
      label: item.label,
      ...(item.description === undefined ? {} : { description: item.description }),
      value: item.value,
    }));
    const picked = await vscode.window.showQuickPick(entries, { placeHolder: placeholder });
    return picked?.value;
  },
};

/**
 * Las carpetas del workspace en disco. Solo `file:`: los espacios de trabajo
 * virtuales no tienen git local, y la extension los declara no soportados.
 */
export const workspaceRepositories: WorkspaceRepositories = {
  list() {
    return (vscode.workspace.workspaceFolders ?? [])
      .filter((folder) => folder.uri.scheme === 'file')
      .map((folder) => ({ name: folder.name, path: folder.uri.fsPath }));
  },
};
