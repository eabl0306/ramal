import {
  type AppearanceStore,
  type StoredAppearance,
  isThemeName,
} from '../../application/ports/appearance-store';

/** Clave en `localStorage`. Lleva version para poder cambiar la forma sin leer basura. */
export const APPEARANCE_KEY = 'ramal.appearance.v1';

/** Lo que se usa de `Storage`; `localStorage` encaja. */
export type KeyValueStorage = Pick<Storage, 'getItem' | 'setItem'>;

/**
 * La apariencia en el `localStorage` del webview, que Tauri guarda por app en el
 * perfil del usuario. Si el almacenamiento falla o trae algo que no se entiende,
 * se ignora: la apariencia por defecto es mejor que no arrancar.
 */
export function createLocalAppearanceStore(storage: KeyValueStorage | null): AppearanceStore {
  return {
    load: () => (storage === null ? {} : read(storage)),
    save: (appearance) => {
      try {
        storage?.setItem(APPEARANCE_KEY, JSON.stringify(appearance));
      } catch {
        // Sin espacio o sin permiso: el ajuste vale para esta sesion y nada mas.
      }
    },
  };
}

function read(storage: KeyValueStorage): StoredAppearance {
  try {
    const raw = storage.getItem(APPEARANCE_KEY);
    return raw === null ? {} : parse(JSON.parse(raw));
  } catch {
    return {};
  }
}

function parse(value: unknown): StoredAppearance {
  if (typeof value !== 'object' || value === null) return {};
  const { theme, locale } = value as Record<string, unknown>;
  return {
    ...(isThemeName(theme) ? { theme } : {}),
    ...(typeof locale === 'string' ? { locale } : {}),
  };
}
