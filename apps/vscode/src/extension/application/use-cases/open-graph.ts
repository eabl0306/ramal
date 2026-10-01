import type { Translate } from '@ramal/i18n';

import { callEngine } from '../call-engine';
import { describeError, errorKey, errorKindOf, type HostVisibleErrorKind } from '../engine-errors';
import type { Dialogs } from '../ports/dialogs';
import type { EngineConnection } from '../ports/engine-connection';
import type { Logger } from '../ports/logger';
import type { Notifications } from '../ports/notifications';
import type { WebviewSurface } from '../ports/webview-surface';
import type { WorkspaceFolderInfo, WorkspaceRepositories } from '../ports/workspace-repositories';
import type { RequestIds } from '../request-ids';

export interface OpenGraphDeps {
  readonly engine: EngineConnection;
  readonly ids: RequestIds;
  readonly repositories: WorkspaceRepositories;
  readonly dialogs: Dialogs;
  readonly notifications: Notifications;
  readonly panels: WebviewSurface;
  readonly translate: Translate;
  readonly logger: Logger;
}

export type OpenGraphResult =
  | { readonly outcome: 'opened'; readonly repoId: string }
  | { readonly outcome: 'no-folder' | 'cancelled' }
  | { readonly outcome: 'failed'; readonly kind: HostVisibleErrorKind };

/**
 * "Ramal: Open Graph" (architecture.md 6.2): elige el repositorio (la carpeta
 * del workspace, o un QuickPick si hay varias), lo abre el propio host con
 * `repo/open` y ensena el panel del grafo con el `repoId`. El webview nunca ve
 * ni manda una ruta. Nunca lanza: los fallos se avisan y se devuelven.
 */
export async function openGraph(deps: OpenGraphDeps): Promise<OpenGraphResult> {
  const folder = await chooseFolder(deps);
  if (folder === 'no-folder' || folder === 'cancelled') return { outcome: folder };
  try {
    const handle = await callEngine(deps.engine, deps.ids, 'repo/open', { path: folder.path });
    deps.panels.show({
      surface: 'graph',
      repoId: handle.repoId,
      title: deps.translate('vscode.tab.graph'),
    });
    return { outcome: 'opened', repoId: handle.repoId };
  } catch (error) {
    deps.logger.warn(`could not open ${folder.name}: ${describeError(error)}`);
    void deps.notifications.showError(deps.translate(errorKey(error)));
    return { outcome: 'failed', kind: errorKindOf(error) };
  }
}

async function chooseFolder(
  deps: OpenGraphDeps,
): Promise<WorkspaceFolderInfo | 'no-folder' | 'cancelled'> {
  const folders = deps.repositories.list();
  const [only] = folders;
  if (only === undefined) {
    deps.notifications.showInfo(deps.translate('vscode.host.noFolder'));
    return 'no-folder';
  }
  if (folders.length === 1) return only;
  const items = folders.map((folder) => ({
    label: folder.name,
    description: folder.path,
    value: folder,
  }));
  const picked = await deps.dialogs.pick(items, deps.translate('vscode.host.pickRepository'));
  return picked ?? 'cancelled';
}
