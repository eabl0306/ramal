import { type BridgeClient, createBridgeClient } from '@git-rudder/bridge';
import { isLocale, resolveLocale } from '@git-rudder/i18n';
import { createRepoViewQueryClient } from '@git-rudder/repo-view';
import type { QueryClient } from '@tanstack/react-query';

import type { AppearanceStore, StoredAppearance } from '../application/ports/appearance-store';
import type { RepositoryPicker } from '../application/ports/repository-picker';
import {
  type KeyValueStorage,
  createLocalAppearanceStore,
} from '../infrastructure/storage/local-appearance-store';
import { type Invoke, tauriIpc } from '../infrastructure/tauri/invoke';
import { createTauriRepositoryPicker } from '../infrastructure/tauri/tauri-repository-picker';
import { createTauriTransport } from '../infrastructure/tauri/tauri-transport';
import type { Appearance } from '../ui/appearance/appearance-context';

/** Lo unico que el escritorio necesita saber del entorno en que arranca. */
export interface DesktopEnv {
  /** El `invoke` de Tauri; los tests pasan uno propio. */
  readonly invoke?: Invoke;
  /** El `localStorage` del webview, o `null` si no se puede usar. */
  readonly storage: KeyValueStorage | null;
  /** El idioma del sistema (`navigator.language`), para el primer arranque. */
  readonly systemLanguage: string;
}

/** Todo lo que la interfaz recibe ya montado. */
export interface Container {
  readonly client: BridgeClient;
  readonly picker: RepositoryPicker;
  readonly appearanceStore: AppearanceStore;
  readonly initialAppearance: Appearance;
  readonly queryClient: QueryClient;
}

/**
 * Raiz de composicion: el unico sitio donde la interfaz y los adaptadores de Tauri
 * se conocen. Cambiar un adaptador se hace aqui y en ningun otro lado.
 */
export function createContainer(env: DesktopEnv): Container {
  const invoke = env.invoke ?? tauriIpc;
  const appearanceStore = createLocalAppearanceStore(env.storage);
  return {
    client: createBridgeClient(createTauriTransport(invoke)),
    picker: createTauriRepositoryPicker(invoke),
    appearanceStore,
    initialAppearance: initialAppearance(appearanceStore.load(), env.systemLanguage),
    queryClient: createRepoViewQueryClient(),
  };
}

/**
 * La apariencia del arranque: lo que el usuario eligio y, para lo que no eligio,
 * tema oscuro (el del diseno) e idioma del sistema con la regla comun de
 * @git-rudder/i18n (castellano si es `es`, ingles en cualquier otro caso).
 */
export function initialAppearance(stored: StoredAppearance, systemLanguage: string): Appearance {
  return {
    theme: stored.theme ?? 'dark',
    locale: isLocale(stored.locale) ? stored.locale : resolveLocale(systemLanguage),
  };
}

/** `localStorage`, o `null` si el webview no deja usarlo (tirar aqui no arrancaria la app). */
export function browserStorage(scope: { readonly localStorage: Storage }): KeyValueStorage | null {
  try {
    return scope.localStorage;
  } catch {
    return null;
  }
}
