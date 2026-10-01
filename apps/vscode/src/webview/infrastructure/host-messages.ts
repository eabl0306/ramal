import {
  type HostFailureResponse,
  type HostToWebviewMessage,
  type InitMessage,
  SURFACES,
} from '../../shared/messages';

/**
 * Lectura de lo que llega por el evento `message`. El host es de fiar (es quien
 * valida al webview), asi que basta con comprobar la forma minima para no
 * romper por un mensaje que no es nuestro.
 */

function isObject(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null;
}

function isInit(value: Record<string, unknown>): value is Record<string, unknown> & InitMessage {
  return (
    (value['locale'] === 'es' || value['locale'] === 'en') &&
    SURFACES.some((surface) => surface === value['surface']) &&
    typeof value['repoId'] === 'string' &&
    typeof value['protocolVersion'] === 'number'
  );
}

function hasField(value: Record<string, unknown>, field: string): boolean {
  return isObject(value[field]);
}

/** El mensaje del host, o `null` si no lo es. */
export function readHostMessage(data: unknown): HostToWebviewMessage | null {
  if (!isObject(data)) return null;
  switch (data['type']) {
    case 'init':
      return isInit(data) ? data : null;
    case 'rpc/response':
      return hasField(data, 'response') ? (data as unknown as HostToWebviewMessage) : null;
    case 'rpc/notification':
      return hasField(data, 'notification') ? (data as unknown as HostToWebviewMessage) : null;
    case 'host/response':
      return typeof data['id'] === 'number' ? (data as unknown as HostToWebviewMessage) : null;
    default:
      return null;
  }
}

/** Una respuesta que fabrico el host porque el motor no podia contestar. */
export function isHostFailure(response: unknown): response is HostFailureResponse {
  if (!isObject(response) || !isObject(response['error'])) return false;
  const data = response['error']['data'];
  return isObject(data) && data['kind'] === 'engine_unavailable';
}
