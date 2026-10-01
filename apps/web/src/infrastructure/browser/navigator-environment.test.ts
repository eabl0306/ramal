import { describe, expect, it } from 'vitest';

import { type BrowserGlobals, createNavigatorEnvironment } from './navigator-environment';

const WINDOWS_UA =
  'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/140.0 Safari/537.36';
const MAC_UA =
  'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/19.0 Safari/605.1.15';

function prefersDark(dark: boolean, light = !dark): NonNullable<BrowserGlobals['matchMedia']> {
  return (query) => ({ matches: query.includes('dark') ? dark : light });
}

describe('createNavigatorEnvironment', () => {
  it('detecta sistema, idioma y tema del navegador', () => {
    const environment = createNavigatorEnvironment(() => ({
      navigator: { userAgent: WINDOWS_UA, language: 'es-AR', maxTouchPoints: 0 },
      matchMedia: prefersDark(false),
    }));

    expect(environment.snapshot()).toEqual({
      platform: { os: 'windows', arch: 'x64' },
      languageTag: 'es-AR',
      systemTheme: 'light',
    });
  });

  it('prefiere userAgentData cuando trae plataforma e ignora una vacia', () => {
    const withHint = createNavigatorEnvironment(() => ({
      navigator: { userAgent: WINDOWS_UA, userAgentData: { platform: 'macOS' } },
    }));
    const emptyHint = createNavigatorEnvironment(() => ({
      navigator: { userAgent: MAC_UA, userAgentData: { platform: ' ' } },
    }));

    expect(withHint.snapshot().platform.os).toBe('mac');
    expect(emptyHint.snapshot().platform.os).toBe('mac');
  });

  it('sin preferencia de color ni idioma deja los dos sin decidir', () => {
    const environment = createNavigatorEnvironment(() => ({
      navigator: { userAgent: MAC_UA, language: '' },
      matchMedia: prefersDark(false, false),
    }));

    expect(environment.snapshot().languageTag).toBeNull();
    expect(environment.snapshot().systemTheme).toBeNull();
  });

  it('con preferencia oscura devuelve el tema oscuro', () => {
    const environment = createNavigatorEnvironment(() => ({
      navigator: { userAgent: MAC_UA },
      matchMedia: prefersDark(true),
    }));

    expect(environment.snapshot().systemTheme).toBe('dark');
  });

  it('sin navegador devuelve un visitante desconocido', () => {
    const environment = createNavigatorEnvironment(() => ({}));

    expect(environment.snapshot()).toEqual({
      platform: { os: 'unknown', arch: 'unknown' },
      languageTag: null,
      systemTheme: null,
    });
  });

  it('lee el navegador una vez y devuelve siempre el mismo objeto', () => {
    let reads = 0;
    const environment = createNavigatorEnvironment(() => {
      reads += 1;
      return { navigator: { userAgent: MAC_UA } };
    });

    expect(environment.snapshot()).toBe(environment.snapshot());
    expect(reads).toBe(1);
  });
});
