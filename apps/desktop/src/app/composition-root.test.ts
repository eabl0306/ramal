import { describe, expect, it, vi } from 'vitest';

import type { Invoke } from '../infrastructure/tauri/invoke';

import { browserStorage, createContainer, initialAppearance } from './composition-root';

describe('initialAppearance', () => {
  it('sin nada guardado: tema oscuro e idioma del sistema', () => {
    expect(initialAppearance({}, 'es-419')).toEqual({ theme: 'dark', locale: 'es' });
    expect(initialAppearance({}, 'pt-BR')).toEqual({ theme: 'dark', locale: 'en' });
  });

  it('lo guardado manda sobre el sistema', () => {
    expect(initialAppearance({ theme: 'light', locale: 'en' }, 'es-ES')).toEqual({
      theme: 'light',
      locale: 'en',
    });
  });

  it('un idioma guardado que ya no existe vuelve al del sistema', () => {
    expect(initialAppearance({ locale: 'fr' }, 'es')).toEqual({ theme: 'dark', locale: 'es' });
  });
});

describe('browserStorage', () => {
  it('devuelve el localStorage si se puede usar', () => {
    expect(browserStorage({ localStorage })).toBe(localStorage);
  });

  it('devuelve null si el webview no deja tocarlo', () => {
    const scope = {
      get localStorage(): Storage {
        throw new Error('SecurityError');
      },
    };

    expect(browserStorage(scope)).toBeNull();
  });
});

describe('createContainer', () => {
  it('cablea el cliente y el selector sobre el mismo invoke', async () => {
    const invoke = vi.fn(() => Promise.resolve(null)) as unknown as Invoke;
    const container = createContainer({ invoke, storage: null, systemLanguage: 'es' });

    await expect(container.picker.pickAndOpen()).resolves.toEqual({ outcome: 'cancelled' });
    expect(invoke).toHaveBeenCalledWith('repo_pick_and_open');
    expect(container.initialAppearance).toEqual({ theme: 'dark', locale: 'es' });
  });
});
