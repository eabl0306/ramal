import type { RpcNotification, RpcResponse } from '@git-rudder/bridge';

/**
 * Comprobaciones minimas de lo que manda el sidecar. El motor es de fiar (va en
 * la misma VSIX y serializa con los mismos tipos que genera ts-rs), pero un
 * marco roto no debe tumbar el host ni resolver la peticion equivocada.
 */

function isObject(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

function isRpcError(value: unknown): boolean {
  return (
    isObject(value) && typeof value['code'] === 'number' && typeof value['message'] === 'string'
  );
}

function isResponseId(value: unknown): boolean {
  return value === null || (typeof value === 'number' && Number.isInteger(value) && value >= 0);
}

/** Respuesta: `jsonrpc`, `id` y o bien `result` o bien un `error` bien formado. */
export function isRpcResponse(value: unknown): value is RpcResponse {
  if (!isObject(value) || value['jsonrpc'] !== '2.0' || !isResponseId(value['id'])) return false;
  const hasResult = 'result' in value;
  const hasError = 'error' in value;
  return hasResult ? !hasError : hasError && isRpcError(value['error']);
}

/** Notificacion: metodo y parametros, sin id. */
export function isRpcNotification(value: unknown): value is RpcNotification {
  return (
    isObject(value) &&
    value['jsonrpc'] === '2.0' &&
    !('id' in value) &&
    typeof value['method'] === 'string' &&
    isObject(value['params'])
  );
}
