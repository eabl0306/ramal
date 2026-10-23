import type { DetectedPlatform } from '@git-rudder/domain';

import type { Theme } from '../../domain/preferences/preferences';

/** Lo que el navegador dice de quien visita, leido una vez por carga. */
export interface VisitorSnapshot {
  readonly platform: DetectedPlatform;
  /** Etiqueta de idioma del navegador (navigator.language), o null si no la hay. */
  readonly languageTag: string | null;
  /** El tema del sistema (prefers-color-scheme), o null si no se sabe. */
  readonly systemTheme: Theme | null;
}

/**
 * Puerto hacia el navegador del visitante. Al construir la landing no hay
 * navegador: la interfaz usa entonces un visitante desconocido y lo cambia por
 * el real al hidratar.
 */
export interface VisitorEnvironment {
  /** Devuelve el mismo objeto en cada llamada. */
  readonly snapshot: () => VisitorSnapshot;
}

export const UNKNOWN_VISITOR: VisitorSnapshot = {
  platform: { os: 'unknown', arch: 'unknown' },
  languageTag: null,
  systemTheme: null,
};
