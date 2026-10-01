import {
  type AuthFieldError,
  type LoginForm,
  type SignupForm,
  validateLogin,
  validateSignup,
} from '@ramal/domain';

import type {
  AuthFailure,
  AuthGateway,
  AuthResult,
  OAuthProvider,
  Session,
} from '../ports/auth-gateway';

/**
 * Casos de uso de entrar y crear cuenta. Validan con @ramal/domain antes de
 * molestar a la API (la API vuelve a validar y es la que manda) y devuelven una
 * union de resultados: nunca lanzan, ni siquiera si el adaptador lo hace.
 */
export type AuthAttempt =
  | { readonly outcome: 'invalid'; readonly errors: readonly AuthFieldError[] }
  | { readonly outcome: 'rejected'; readonly failure: AuthFailure }
  | { readonly outcome: 'signed-in'; readonly session: Session };

export interface SignupInput extends SignupForm {
  /** Opcional: un texto vacio no crea organizacion. */
  readonly organization: string;
}

async function settle(call: () => Promise<AuthResult>): Promise<AuthAttempt> {
  try {
    const result = await call();
    return result.ok
      ? { outcome: 'signed-in', session: result.session }
      : { outcome: 'rejected', failure: result.failure };
  } catch {
    return { outcome: 'rejected', failure: 'network' };
  }
}

export function signIn(gateway: AuthGateway, form: LoginForm): Promise<AuthAttempt> {
  const validation = validateLogin(form);
  if (!validation.ok) return Promise.resolve({ outcome: 'invalid', errors: validation.errors });
  return settle(() => gateway.signIn({ email: form.email.trim(), password: form.password }));
}

export function signUp(gateway: AuthGateway, form: SignupInput): Promise<AuthAttempt> {
  const validation = validateSignup(form);
  if (!validation.ok) return Promise.resolve({ outcome: 'invalid', errors: validation.errors });
  const organization = form.organization.trim();
  return settle(() =>
    gateway.signUp({
      name: form.name.trim(),
      email: form.email.trim(),
      password: form.password,
      organization: organization === '' ? null : organization,
    }),
  );
}

/** Con un proveedor no hay formulario que validar: los terminos los pedira su vuelta (W-04). */
export function signInWithProvider(
  gateway: AuthGateway,
  provider: OAuthProvider,
): Promise<AuthAttempt> {
  return settle(() => gateway.signInWithProvider(provider));
}
