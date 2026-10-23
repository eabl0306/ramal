import { describe, expect, it } from 'vitest';

import { isValidEmail, passwordHint, validateLogin, validateSignup } from './auth-form';

const VALID_SIGNUP = {
  name: 'Marta Ruiz',
  email: 'marta@example.com',
  password: '12345678',
  termsAccepted: true,
};

describe('isValidEmail', () => {
  it('acepta lo que acepta el prototipo y recorta espacios', () => {
    expect(isValidEmail('marta@example.com')).toBe(true);
    expect(isValidEmail('  marta@example.com ')).toBe(true);
  });

  it('rechaza emails sin arroba, sin punto o con espacios dentro', () => {
    expect(isValidEmail('marta.example.com')).toBe(false);
    expect(isValidEmail('marta@git-rudder')).toBe(false);
    expect(isValidEmail('marta @example.com')).toBe(false);
    expect(isValidEmail('')).toBe(false);
  });
});

describe('passwordHint', () => {
  it('cambia a ok justo en 8 caracteres', () => {
    expect(passwordHint('1234567')).toBe('too-short');
    expect(passwordHint('12345678')).toBe('ok');
  });
});

describe('validateSignup', () => {
  it('un formulario completo es valido', () => {
    expect(validateSignup(VALID_SIGNUP)).toEqual({ ok: true });
  });

  it('devuelve los errores en el orden de los campos', () => {
    expect(
      validateSignup({ name: '   ', email: 'x', password: 'corta', termsAccepted: false }),
    ).toEqual({
      ok: false,
      errors: [
        { field: 'name', code: 'required' },
        { field: 'email', code: 'invalid' },
        { field: 'password', code: 'too-short' },
        { field: 'terms', code: 'required' },
      ],
    });
  });

  it('los terminos son obligatorios', () => {
    expect(validateSignup({ ...VALID_SIGNUP, termsAccepted: false })).toEqual({
      ok: false,
      errors: [{ field: 'terms', code: 'required' }],
    });
  });
});

describe('validateLogin', () => {
  it('no aplica el minimo de 8 caracteres al entrar', () => {
    expect(validateLogin({ email: 'marta@example.com', password: 'abc' })).toEqual({ ok: true });
  });

  it('pide un email valido y una contrasena no vacia', () => {
    expect(validateLogin({ email: 'marta', password: '' })).toEqual({
      ok: false,
      errors: [
        { field: 'email', code: 'invalid' },
        { field: 'password', code: 'required' },
      ],
    });
  });
});
