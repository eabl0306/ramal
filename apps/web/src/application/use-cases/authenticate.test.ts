import { describe, expect, it, vi } from 'vitest';

import type { AuthGateway, AuthResult } from '../ports/auth-gateway';

import { signIn, signInWithProvider, signUp } from './authenticate';

const UNAVAILABLE: AuthResult = { ok: false, failure: 'unavailable' };

/** Puerto escrito a mano: todas las llamadas contestan lo mismo y quedan apuntadas. */
function gatewayAnswering(result: AuthResult): AuthGateway {
  return {
    signIn: vi.fn(() => Promise.resolve(result)),
    signUp: vi.fn(() => Promise.resolve(result)),
    signInWithProvider: vi.fn(() => Promise.resolve(result)),
  };
}

const VALID_SIGNUP = {
  name: ' Marta Ruiz ',
  email: ' marta@ramal.dev ',
  password: 'contrasena-larga',
  termsAccepted: true,
  organization: '  ',
};

describe('signIn', () => {
  it('no llama a la API si el formulario no es valido y devuelve los errores en orden', async () => {
    const gateway = gatewayAnswering(UNAVAILABLE);

    const attempt = await signIn(gateway, { email: 'no-es-un-email', password: '' });

    expect(attempt).toEqual({
      outcome: 'invalid',
      errors: [
        { field: 'email', code: 'invalid' },
        { field: 'password', code: 'required' },
      ],
    });
    expect(gateway.signIn).not.toHaveBeenCalled();
  });

  it('no exige la longitud minima al entrar y recorta el email', async () => {
    const gateway = gatewayAnswering({ ok: true, session: { email: 'marta@ramal.dev' } });

    const attempt = await signIn(gateway, { email: ' marta@ramal.dev ', password: 'corta' });

    expect(attempt).toEqual({ outcome: 'signed-in', session: { email: 'marta@ramal.dev' } });
    expect(gateway.signIn).toHaveBeenCalledWith({ email: 'marta@ramal.dev', password: 'corta' });
  });

  it('devuelve el rechazo del adaptador tal cual', async () => {
    const attempt = await signIn(gatewayAnswering(UNAVAILABLE), {
      email: 'marta@ramal.dev',
      password: 'x',
    });

    expect(attempt).toEqual({ outcome: 'rejected', failure: 'unavailable' });
  });

  it('convierte una excepcion del adaptador en un fallo de red', async () => {
    const gateway: AuthGateway = {
      ...gatewayAnswering(UNAVAILABLE),
      signIn: () => Promise.reject(new Error('sin red')),
    };

    const attempt = await signIn(gateway, { email: 'marta@ramal.dev', password: 'x' });

    expect(attempt).toEqual({ outcome: 'rejected', failure: 'network' });
  });
});

describe('signUp', () => {
  it('valida nombre, email, contrasena y terminos antes de llamar', async () => {
    const gateway = gatewayAnswering(UNAVAILABLE);

    const attempt = await signUp(gateway, {
      name: '',
      email: 'x',
      password: 'corta',
      termsAccepted: false,
      organization: '',
    });

    expect(attempt.outcome).toBe('invalid');
    expect(attempt.outcome === 'invalid' && attempt.errors.map((error) => error.field)).toEqual([
      'name',
      'email',
      'password',
      'terms',
    ]);
    expect(gateway.signUp).not.toHaveBeenCalled();
  });

  it('recorta los textos y no manda una organizacion vacia', async () => {
    const gateway = gatewayAnswering(UNAVAILABLE);

    await signUp(gateway, VALID_SIGNUP);

    expect(gateway.signUp).toHaveBeenCalledWith({
      name: 'Marta Ruiz',
      email: 'marta@ramal.dev',
      password: 'contrasena-larga',
      organization: null,
    });
  });

  it('manda la organizacion cuando se escribe', async () => {
    const gateway = gatewayAnswering(UNAVAILABLE);

    await signUp(gateway, { ...VALID_SIGNUP, organization: ' ACME Studio ' });

    expect(gateway.signUp).toHaveBeenCalledWith(
      expect.objectContaining({ organization: 'ACME Studio' }),
    );
  });
});

describe('signInWithProvider', () => {
  it('pasa el proveedor al adaptador y devuelve su respuesta', async () => {
    const gateway = gatewayAnswering(UNAVAILABLE);

    const attempt = await signInWithProvider(gateway, 'github');

    expect(attempt).toEqual({ outcome: 'rejected', failure: 'unavailable' });
    expect(gateway.signInWithProvider).toHaveBeenCalledWith('github');
  });
});
