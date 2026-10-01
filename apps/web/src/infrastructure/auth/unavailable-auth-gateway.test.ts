import { describe, expect, it } from 'vitest';

import { createUnavailableAuthGateway } from './unavailable-auth-gateway';

describe('createUnavailableAuthGateway', () => {
  it('contesta no disponible a entrar, crear cuenta y entrar con un proveedor', async () => {
    const gateway = createUnavailableAuthGateway();
    const unavailable = { ok: false, failure: 'unavailable' };

    await expect(gateway.signIn({ email: 'a@b.co', password: 'x' })).resolves.toEqual(unavailable);
    await expect(
      gateway.signUp({ name: 'A', email: 'a@b.co', password: 'x', organization: null }),
    ).resolves.toEqual(unavailable);
    await expect(gateway.signInWithProvider('github')).resolves.toEqual(unavailable);
  });
});
