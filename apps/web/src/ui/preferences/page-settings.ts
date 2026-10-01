import { resolveLocale } from '@ramal/i18n';
import { useSyncExternalStore } from 'react';
import { useLocation } from 'react-router';

import type { PreferencesStore } from '../../application/ports/preferences-store';
import {
  UNKNOWN_VISITOR,
  type VisitorEnvironment,
  type VisitorSnapshot,
} from '../../application/ports/visitor-environment';
import { effectiveTheme, pageLanguage } from '../../domain/preferences/page-preferences';
import {
  DEFAULT_LANGUAGE,
  type Language,
  NO_PREFERENCES,
  type StoredPreferences,
  type Theme,
} from '../../domain/preferences/preferences';

/**
 * Idioma y tema de la pagina. Al prerenderizar no hay navegador ni preferencias
 * guardadas, asi que el servidor pinta los valores por defecto; al hidratar,
 * useSyncExternalStore repite esos mismos valores (los del servidor) y despues
 * cambia a los del navegador sin que React vea un HTML distinto del suyo.
 */

const serverPreferences = (): StoredPreferences => NO_PREFERENCES;
const serverVisitor = (): VisitorSnapshot => UNKNOWN_VISITOR;
const noSubscription = (): (() => void) => () => undefined;

export function useStoredPreferences(store: PreferencesStore): StoredPreferences {
  return useSyncExternalStore(store.subscribe, store.read, serverPreferences);
}

/** El navegador no cambia durante la visita: no hay a que suscribirse. */
export function useVisitorSnapshot(environment: VisitorEnvironment): VisitorSnapshot {
  return useSyncExternalStore(noSubscription, environment.snapshot, serverVisitor);
}

export interface PageSettings {
  readonly language: Language;
  readonly theme: Theme;
}

function preferredLanguage(stored: StoredPreferences, visitor: VisitorSnapshot): Language {
  if (stored.language !== null) return stored.language;
  return visitor.languageTag === null ? DEFAULT_LANGUAGE : resolveLocale(visitor.languageTag);
}

export function usePageSettings(
  store: PreferencesStore,
  environment: VisitorEnvironment,
): PageSettings {
  const stored = useStoredPreferences(store);
  const visitor = useVisitorSnapshot(environment);
  const { pathname } = useLocation();
  return {
    language: pageLanguage(pathname, preferredLanguage(stored, visitor)),
    theme: effectiveTheme(stored.theme, visitor.systemTheme),
  };
}
