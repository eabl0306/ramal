import { describe, expect, it } from 'vitest';

import { createContainer, resolveApiUrl } from './composition-root';

describe('resolveApiUrl', () => {
  it('sin VITE_API_URL usa el mismo origen', () => {
    expect(resolveApiUrl({})).toBe('');
  });

  it('recorta la URL configurada', () => {
    expect(resolveApiUrl({ VITE_API_URL: ' https://api.ramal.dev ' })).toBe(
      'https://api.ramal.dev',
    );
  });
});

describe('createContainer', () => {
  it('monta todos los puertos sin tocar el navegador', async () => {
    const container = createContainer({});

    expect(Object.keys(container).sort()).toEqual(
      ['auth', 'clipboard', 'health', 'preferences', 'releases', 'visitor'].sort(),
    );
    await expect(container.auth.signIn({ email: 'a@b.co', password: 'x' })).resolves.toEqual({
      ok: false,
      failure: 'unavailable',
    });
    await expect(container.releases.latest()).resolves.toHaveProperty('version');
  });
});
