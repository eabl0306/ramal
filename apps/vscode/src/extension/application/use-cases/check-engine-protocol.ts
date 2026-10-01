import { type EngineInfo, PROTOCOL_VERSION } from '@ramal/bridge';

import { callEngine } from '../call-engine';
import type { EngineConnection } from '../ports/engine-connection';
import type { RequestIds } from '../request-ids';

export type ProtocolCheck =
  | { readonly outcome: 'compatible'; readonly info: EngineInfo }
  | { readonly outcome: 'mismatch'; readonly engineVersion: number; readonly expected: number }
  | { readonly outcome: 'failed'; readonly error: unknown };

/**
 * Pregunta `engine/info` a un motor recien arrancado y compara su version del
 * protocolo con la que trae generada la extension. Los dos viajan en la misma
 * VSIX, asi que una diferencia es un fallo de empaquetado, no algo que se
 * arregle reintentando. Nunca lanza: devuelve el resultado.
 */
export async function checkEngineProtocol(
  connection: EngineConnection,
  ids: RequestIds,
  expected: number = PROTOCOL_VERSION,
): Promise<ProtocolCheck> {
  try {
    const info = await callEngine(connection, ids, 'engine/info', {});
    if (info.protocolVersion !== expected) {
      return { outcome: 'mismatch', engineVersion: info.protocolVersion, expected };
    }
    return { outcome: 'compatible', info };
  } catch (error) {
    return { outcome: 'failed', error };
  }
}
