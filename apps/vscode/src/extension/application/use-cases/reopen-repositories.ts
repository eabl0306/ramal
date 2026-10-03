import type { Translate } from '@git-rudder/i18n';

import { callEngine } from '../call-engine';
import { EngineCallError, describeError, errorKey } from '../engine-errors';
import type { EngineConnection } from '../ports/engine-connection';
import type { Logger } from '../ports/logger';
import type { Notifications } from '../ports/notifications';
import type { WebviewSurface } from '../ports/webview-surface';
import type { RequestIds } from '../request-ids';

export interface ReopenRepositoriesDeps {
  readonly ids: RequestIds;
  readonly panels: WebviewSurface;
  readonly notifications: Notifications;
  readonly translate: Translate;
  readonly logger: Logger;
}

/**
 * Tras reiniciar el sidecar (architecture.md 4.4) su registro de repositorios
 * esta vacio. Existe por un fallo real: nadie los volvia a abrir y cada panel
 * abierto se quedaba en `repo_not_open` para siempre. Abre otra vez con
 * `repo/open` la raiz de cada panel sobre la conexion del proceso nuevo, antes
 * de que el supervisor deje pasar ninguna peticion de los webviews.
 *
 * Si el motor contesta con error (la carpeta ya no es un repositorio), se
 * cierran sus paneles y se avisa. Si el motor no contesta, no se toca nada: se
 * ha vuelto a caer y el siguiente arranque lo intentara otra vez. Nunca lanza.
 */
export async function reopenRepositories(
  connection: EngineConnection,
  deps: ReopenRepositoriesDeps,
): Promise<void> {
  await Promise.all(deps.panels.openRoots().map((root) => reopen(connection, deps, root)));
}

async function reopen(
  connection: EngineConnection,
  deps: ReopenRepositoriesDeps,
  root: string,
): Promise<void> {
  try {
    const handle = await callEngine(connection, deps.ids, 'repo/open', { path: root });
    deps.panels.rebind(root, handle.repoId);
  } catch (error) {
    deps.logger.warn(`could not reopen ${root} after the restart: ${describeError(error)}`);
    if (!(error instanceof EngineCallError)) return;
    deps.panels.close(root);
    void deps.notifications.showError(deps.translate(errorKey(error)));
  }
}
