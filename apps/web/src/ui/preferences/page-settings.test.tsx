import { act, renderHook } from '@testing-library/react';
import type { ReactElement, ReactNode } from 'react';
import { MemoryRouter } from 'react-router';
import { describe, expect, it } from 'vitest';

import type { VisitorEnvironment } from '../../application/ports/visitor-environment';
import { testPorts } from '../../test/render';

import { usePageSettings } from './page-settings';

function visitor(
  languageTag: string | null,
  systemTheme: 'dark' | 'light' | null,
): VisitorEnvironment {
  const snapshot = { platform: { os: 'linux', arch: 'x64' } as const, languageTag, systemTheme };
  return { snapshot: () => snapshot };
}

function routerAt(path: string) {
  return function Wrapper({ children }: { readonly children: ReactNode }): ReactElement {
    return <MemoryRouter initialEntries={[path]}>{children}</MemoryRouter>;
  };
}

describe('usePageSettings', () => {
  it('en la landing el idioma sale de la URL aunque el navegador diga otro', () => {
    const { preferences } = testPorts();
    const { result } = renderHook(() => usePageSettings(preferences, visitor('es-ES', null)), {
      wrapper: routerAt('/en/'),
    });

    expect(result.current).toEqual({ language: 'en', theme: 'dark' });
  });

  it('fuera de la landing usa el idioma del navegador si no hay preferencia', () => {
    const { preferences } = testPorts();
    const { result } = renderHook(() => usePageSettings(preferences, visitor('en-GB', 'light')), {
      wrapper: routerAt('/status'),
    });

    expect(result.current).toEqual({ language: 'en', theme: 'light' });
  });

  it('sin preferencia ni idioma del navegador habla castellano', () => {
    const { preferences } = testPorts();
    const { result } = renderHook(() => usePageSettings(preferences, visitor(null, null)), {
      wrapper: routerAt('/login'),
    });

    expect(result.current.language).toBe('es');
  });

  it('lo guardado gana y un cambio se ve sin recargar', () => {
    const { preferences } = testPorts();
    const { result } = renderHook(() => usePageSettings(preferences, visitor('en-US', 'dark')), {
      wrapper: routerAt('/login'),
    });

    act(() => {
      preferences.save({ language: 'es', theme: 'light' });
    });

    expect(result.current).toEqual({ language: 'es', theme: 'light' });
  });
});
