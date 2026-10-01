import { describe, expect, it } from 'vitest';

import { APPEARANCE_KEY, createLocalAppearanceStore } from './local-appearance-store';

function memoryStorage(initial: Record<string, string> = {}): Storage {
  const values = new Map(Object.entries(initial));
  return {
    get length() {
      return values.size;
    },
    clear: () => {
      values.clear();
    },
    key: (index) => [...values.keys()][index] ?? null,
    getItem: (key) => values.get(key) ?? null,
    setItem: (key, value) => {
      values.set(key, value);
    },
    removeItem: (key) => {
      values.delete(key);
    },
  };
}

const failingStorage: Pick<Storage, 'getItem' | 'setItem'> = {
  getItem: () => {
    throw new Error('SecurityError');
  },
  setItem: () => {
    throw new Error('QuotaExceededError');
  },
};

describe('createLocalAppearanceStore', () => {
  it('devuelve vacio la primera vez', () => {
    expect(createLocalAppearanceStore(memoryStorage()).load()).toEqual({});
  });

  it('guarda y vuelve a leer el tema y el idioma', () => {
    const storage = memoryStorage();
    const store = createLocalAppearanceStore(storage);

    store.save({ theme: 'light', locale: 'en' });

    expect(storage.getItem(APPEARANCE_KEY)).toBe('{"theme":"light","locale":"en"}');
    expect(createLocalAppearanceStore(storage).load()).toEqual({ theme: 'light', locale: 'en' });
  });

  it('descarta un tema que no existe y conserva el idioma', () => {
    const storage = memoryStorage({ [APPEARANCE_KEY]: '{"theme":"sepia","locale":"es"}' });

    expect(createLocalAppearanceStore(storage).load()).toEqual({ locale: 'es' });
  });

  it.each(['no es json', 'null', '42', '{"locale":7}'])('ignora lo que no entiende: %s', (raw) => {
    const storage = memoryStorage({ [APPEARANCE_KEY]: raw });

    expect(createLocalAppearanceStore(storage).load()).toEqual({});
  });

  it('sigue funcionando si el almacenamiento falla o no existe', () => {
    const store = createLocalAppearanceStore(failingStorage);

    expect(store.load()).toEqual({});
    expect(() => {
      store.save({ theme: 'dark', locale: 'es' });
    }).not.toThrow();
    expect(createLocalAppearanceStore(null).load()).toEqual({});
  });
});
