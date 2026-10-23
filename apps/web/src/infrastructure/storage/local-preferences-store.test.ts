import { describe, expect, it, vi } from 'vitest';

import { createLocalPreferencesStore } from './local-preferences-store';

function memoryStorage(initial: Record<string, string> = {}) {
  const items = new Map(Object.entries(initial));
  return {
    items,
    getItem: (key: string) => items.get(key) ?? null,
    setItem: (key: string, value: string) => {
      items.set(key, value);
    },
  };
}

describe('createLocalPreferencesStore', () => {
  it('sin nada guardado no hay preferencias', () => {
    const store = createLocalPreferencesStore(() => memoryStorage());

    expect(store.read()).toEqual({ language: null, theme: null });
  });

  it('lee lo guardado e ignora valores desconocidos campo a campo', () => {
    const storage = memoryStorage({
      'gitRudder.web.preferences': JSON.stringify({ language: 'en', theme: 'sepia' }),
    });
    const store = createLocalPreferencesStore(() => storage);

    expect(store.read()).toEqual({ language: 'en', theme: null });
  });

  it('ignora un JSON roto o que no es un objeto', () => {
    const broken = createLocalPreferencesStore(() =>
      memoryStorage({ 'gitRudder.web.preferences': '{' }),
    );
    const scalar = createLocalPreferencesStore(() =>
      memoryStorage({ 'gitRudder.web.preferences': '3' }),
    );

    expect(broken.read()).toEqual({ language: null, theme: null });
    expect(scalar.read()).toEqual({ language: null, theme: null });
  });

  it('guarda, avisa a los suscritos y devuelve el mismo objeto hasta el siguiente cambio', () => {
    const storage = memoryStorage();
    const store = createLocalPreferencesStore(() => storage);
    const listener = vi.fn();
    const unsubscribe = store.subscribe(listener);

    store.save({ theme: 'light' });

    expect(listener).toHaveBeenCalledTimes(1);
    expect(store.read()).toBe(store.read());
    expect(store.read()).toEqual({ language: null, theme: 'light' });
    expect(JSON.parse(storage.items.get('gitRudder.web.preferences') ?? '')).toEqual({
      language: null,
      theme: 'light',
    });

    unsubscribe();
    store.save({ language: 'es' });
    expect(listener).toHaveBeenCalledTimes(1);
  });

  it('sigue en memoria si el navegador bloquea el almacenamiento', () => {
    const blocked = {
      getItem: () => {
        throw new Error('SecurityError');
      },
      setItem: () => {
        throw new Error('QuotaExceededError');
      },
    };
    const store = createLocalPreferencesStore(() => blocked);

    expect(store.read()).toEqual({ language: null, theme: null });
    store.save({ language: 'en' });
    expect(store.read().language).toBe('en');
  });

  it('sin localStorage, como al construir la landing, no falla', () => {
    const store = createLocalPreferencesStore(() => undefined);

    store.save({ theme: 'dark' });
    expect(store.read().theme).toBe('dark');
  });
});
