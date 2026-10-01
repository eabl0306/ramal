import type { GraphWindow, GraphWindowParams, RpcRequest, RpcResponse } from '@ramal/bridge';
import { mockIPC } from '@tauri-apps/api/mocks';

import type { E2eCall, E2eScenario } from './scenario';

/**
 * El host de Tauri simulado dentro de la pagina con `mockIPC` de
 * @tauri-apps/api/mocks. Global-setup lo empaqueta como script y cada prueba lo
 * inyecta antes de cargar la app: contesta `repo_pick_and_open` y `rpc` como
 * src-tauri, segun el escenario de window.__RAMAL_E2E__, y apunta cada llamada en
 * window.__RAMAL_E2E_CALLS__ para que la prueba vea que pidio el webview.
 */
mockIPC((command, payload) => {
  const scenario = window.__RAMAL_E2E__;
  if (scenario === undefined) throw new Error('missing window.__RAMAL_E2E__ scenario');
  const request = (payload as { request?: RpcRequest } | undefined)?.request;
  record({ command, ...(request === undefined ? {} : { method: request.method }) });
  if (command === 'repo_pick_and_open') return pickAndOpen(scenario);
  if (command === 'rpc' && request !== undefined) return rpc(scenario, request);
  throw new Error(`command ${command} not allowed`);
});

function record(call: E2eCall): void {
  window.__RAMAL_E2E_CALLS__ = [...(window.__RAMAL_E2E_CALLS__ ?? []), call];
}

function pickAndOpen({ pick }: E2eScenario): unknown {
  if (pick.outcome === 'opened') return pick.repo;
  if (pick.outcome === 'cancelled') return null;
  // Como Tauri: el comando rechaza con el RpcError serializado del motor, un
  // objeto plano y no un Error.
  // eslint-disable-next-line @typescript-eslint/only-throw-error
  throw pick.error;
}

function rpc({ rows }: E2eScenario, request: RpcRequest): RpcResponse | null {
  if (request.id === undefined) return null;
  if (request.method !== 'graph/window') {
    return {
      jsonrpc: '2.0',
      id: request.id,
      error: { code: -32_601, message: 'not available', data: { kind: 'internal' } },
    };
  }
  const { repoId, offset, limit } = request.params as GraphWindowParams;
  const result: GraphWindow = {
    repoId,
    offset,
    rows: rows.slice(offset, offset + limit),
    hasMore: offset + limit < rows.length,
    laneCount: 2,
    snapshot: 'e2e',
  };
  return { jsonrpc: '2.0', id: request.id, result };
}
