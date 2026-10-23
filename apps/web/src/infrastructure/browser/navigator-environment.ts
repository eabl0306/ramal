import { type PlatformHints, detectPlatform } from '@git-rudder/domain';

import {
  UNKNOWN_VISITOR,
  type VisitorEnvironment,
  type VisitorSnapshot,
} from '../../application/ports/visitor-environment';
import type { Theme } from '../../domain/preferences/preferences';

/** Lo poco que se lee de navigator. userAgentData no esta en la lib DOM de TypeScript. */
interface NavigatorLike {
  readonly userAgent: string;
  readonly maxTouchPoints?: number;
  readonly language?: string;
  readonly userAgentData?: { readonly platform?: string };
}

export interface BrowserGlobals {
  readonly navigator?: NavigatorLike;
  readonly matchMedia?: (query: string) => { readonly matches: boolean };
}

const DARK_QUERY = '(prefers-color-scheme: dark)';
const LIGHT_QUERY = '(prefers-color-scheme: light)';

/** Una pista vacia es lo mismo que ninguna: algunos navegadores dan platform "". */
function platformHints(navigator: NavigatorLike): PlatformHints {
  const uaPlatform = navigator.userAgentData?.platform?.trim() ?? '';
  return {
    userAgent: navigator.userAgent,
    maxTouchPoints: navigator.maxTouchPoints ?? 0,
    ...(uaPlatform === '' ? {} : { uaPlatform }),
  };
}

function systemTheme(globals: BrowserGlobals): Theme | null {
  if (globals.matchMedia === undefined) return null;
  if (globals.matchMedia(DARK_QUERY).matches) return 'dark';
  return globals.matchMedia(LIGHT_QUERY).matches ? 'light' : null;
}

function read(globals: BrowserGlobals): VisitorSnapshot {
  const { navigator } = globals;
  if (navigator === undefined) return UNKNOWN_VISITOR;
  const language = navigator.language?.trim() ?? '';
  return {
    platform: detectPlatform(platformHints(navigator)),
    languageTag: language === '' ? null : language,
    systemTheme: systemTheme(globals),
  };
}

/**
 * Lee el navegador una sola vez y devuelve siempre el mismo objeto, que es lo
 * que necesita useSyncExternalStore para no volver a pintar sin motivo.
 */
export function createNavigatorEnvironment(
  globals: () => BrowserGlobals = () => globalThis,
): VisitorEnvironment {
  let cached: VisitorSnapshot | null = null;
  return {
    snapshot: () => {
      cached ??= read(globals());
      return cached;
    },
  };
}
