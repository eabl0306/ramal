import { invoke as tauriInvoke } from '@tauri-apps/api/core';

/**
 * La forma de `invoke` que usan los adaptadores. Se inyecta para que los tests
 * pasen una funcion propia sin `window.__TAURI_INTERNALS__`.
 */
export type Invoke = <T>(command: string, args?: Record<string, unknown>) => Promise<T>;

/** El `invoke` de verdad, el de `@tauri-apps/api/core`. */
export const tauriIpc: Invoke = (command, args) => tauriInvoke(command, args);
