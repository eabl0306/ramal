import type { EngineMethod, EngineMethods } from '@ramal/bridge';

import { callEngine } from '../call-engine';
import type { EngineConnection } from '../ports/engine-connection';
import type { RequestIds } from '../request-ids';

import type { RoutedRequest, WebviewRouter } from './route-webview-message';

/**
 * Lo que la extension expone a los tests de integracion, y solo con
 * `ExtensionMode.Test`: ver que peticiones de los webviews llegaron al motor y
 * llamarlo como lo haria el host. No es una API publica; en produccion
 * `activate` no devuelve nada.
 */
export interface HostProbe {
  routed(): readonly RoutedRequest[];
  call<M extends EngineMethod>(
    method: M,
    params: EngineMethods[M]['params'],
  ): Promise<EngineMethods[M]['result']>;
}

export function createHostProbe(
  router: Pick<WebviewRouter, 'onRouted'>,
  engine: EngineConnection,
  ids: RequestIds,
): HostProbe {
  const routed: RoutedRequest[] = [];
  router.onRouted((entry) => {
    routed.push(entry);
  });
  return {
    routed: () => [...routed],
    call: (method, params) => callEngine(engine, ids, method, params),
  };
}
