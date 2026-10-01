import { type BridgeErrorKind, type RepoHandle, isBridgeErrorKind } from '@ramal/bridge';

import type {
  PickRepositoryResult,
  RepositoryPicker,
} from '../../application/ports/repository-picker';

import { type Invoke, tauriIpc } from './invoke';

/** Comando de Tauri que ensena el dialogo de carpetas y abre el repositorio. */
export const PICK_AND_OPEN_COMMAND = 'repo_pick_and_open';

/**
 * `RepositoryPicker` sobre `repo_pick_and_open`. El comando devuelve el
 * `RepoHandle`, `null` si el usuario cancela, o rechaza con el `RpcError` del motor
 * (`{ code, message, data: { kind } }`). Cualquier otro rechazo es un fallo del
 * transporte.
 */
export function createTauriRepositoryPicker(invoke: Invoke = tauriIpc): RepositoryPicker {
  return {
    async pickAndOpen(): Promise<PickRepositoryResult> {
      try {
        const repo = await invoke<RepoHandle | null>(PICK_AND_OPEN_COMMAND);
        return repo === null ? { outcome: 'cancelled' } : { outcome: 'opened', repo };
      } catch (error) {
        return { outcome: 'failed', kind: errorKind(error) };
      }
    },
  };
}

/** El `kind` del `RpcError` rechazado, o `transport` si no tiene esa forma. */
export function errorKind(error: unknown): BridgeErrorKind {
  if (typeof error !== 'object' || error === null || !('data' in error)) return 'transport';
  const { data } = error;
  if (typeof data !== 'object' || data === null || !('kind' in data)) return 'transport';
  return isBridgeErrorKind(data.kind) ? data.kind : 'internal';
}
