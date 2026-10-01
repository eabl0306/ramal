import type { StoredPreferences } from '../../domain/preferences/preferences';

/**
 * Donde la web recuerda el idioma y el tema elegidos. Tiene forma de almacen
 * externo (leer, guardar, suscribirse) para que React lo lea con
 * useSyncExternalStore: lectura estable mientras nada cambie.
 */
export interface PreferencesStore {
  /** Devuelve el mismo objeto mientras no haya cambios. */
  readonly read: () => StoredPreferences;
  readonly save: (change: Partial<StoredPreferences>) => void;
  /** Devuelve la funcion que da de baja al oyente. */
  readonly subscribe: (listener: () => void) => () => void;
}
