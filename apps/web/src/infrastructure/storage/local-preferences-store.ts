import type { PreferencesStore } from '../../application/ports/preferences-store';
import {
  NO_PREFERENCES,
  type StoredPreferences,
  isLanguage,
  isTheme,
} from '../../domain/preferences/preferences';

const STORAGE_KEY = 'ramal.web.preferences';

/** De donde sale el almacenamiento; los tests pasan uno en memoria. */
export type StorageSource = () => Pick<Storage, 'getItem' | 'setItem'> | undefined;

const browserStorage: StorageSource = () =>
  typeof localStorage === 'undefined' ? undefined : localStorage;

/** Lo que haya guardado otra version de la web o a mano se ignora campo a campo. */
function parse(raw: string | null): StoredPreferences {
  if (raw === null) return NO_PREFERENCES;
  try {
    const value: unknown = JSON.parse(raw);
    if (typeof value !== 'object' || value === null) return NO_PREFERENCES;
    const record = value as Record<string, unknown>;
    return {
      language: isLanguage(record['language']) ? record['language'] : null,
      theme: isTheme(record['theme']) ? record['theme'] : null,
    };
  } catch {
    return NO_PREFERENCES;
  }
}

/**
 * Preferencias en localStorage. Safari en privado y los navegadores con el
 * almacenamiento bloqueado lanzan al leer o escribir: la web sigue funcionando
 * con lo que tenga en memoria. Al construir la landing no hay localStorage, y
 * por eso nada se lee hasta la primera llamada.
 */
export function createLocalPreferencesStore(
  source: StorageSource = browserStorage,
): PreferencesStore {
  const listeners = new Set<() => void>();
  let current: StoredPreferences | null = null;

  const read = (): StoredPreferences => {
    if (current !== null) return current;
    try {
      current = parse(source()?.getItem(STORAGE_KEY) ?? null);
    } catch {
      current = NO_PREFERENCES;
    }
    return current;
  };

  const save = (change: Partial<StoredPreferences>): void => {
    current = { ...read(), ...change };
    try {
      source()?.setItem(STORAGE_KEY, JSON.stringify(current));
    } catch {
      // Sin almacenamiento la eleccion dura hasta recargar, que es lo mejor posible.
    }
    for (const listener of listeners) listener();
  };

  const subscribe = (listener: () => void): (() => void) => {
    listeners.add(listener);
    return () => {
      listeners.delete(listener);
    };
  };

  return { read, save, subscribe };
}
