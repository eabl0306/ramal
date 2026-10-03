import type { EngineMethod, EngineMethods } from '@git-rudder/bridge';

import { EngineCallError } from './engine-errors';
import type { EngineConnection } from './ports/engine-connection';
import type { RequestIds } from './request-ids';

/**
 * Llamada del propio host al motor (`engine/info`, `repo/open`), tipada por el
 * mapa de metodos generado. Rechaza con `EngineCallError` si el motor contesta
 * con error y con `EngineUnavailableError` si no esta.
 */
export async function callEngine<M extends EngineMethod>(
  engine: EngineConnection,
  ids: RequestIds,
  method: M,
  params: EngineMethods[M]['params'],
): Promise<EngineMethods[M]['result']> {
  const response = await engine.request({ jsonrpc: '2.0', id: ids.next(), method, params });
  if ('error' in response) throw new EngineCallError(response.error);
  // El motor y los tipos salen del mismo `define_methods!`: el resultado de `M`
  // tiene la forma de `EngineMethods[M]['result']` por construccion.
  return response.result as EngineMethods[M]['result'];
}
