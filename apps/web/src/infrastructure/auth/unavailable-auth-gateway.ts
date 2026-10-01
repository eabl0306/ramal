import type { AuthGateway, AuthResult } from '../../application/ports/auth-gateway';

const UNAVAILABLE: AuthResult = { ok: false, failure: 'unavailable' };

/**
 * El unico adaptador de AuthGateway del primer corte. La API todavia no tiene el
 * contexto auth (architecture.md 10), asi que entrar y crear cuenta contestan
 * "todavia no esta disponible" despues de validar el formulario. Cuando exista
 * el endpoint, entra un adaptador HTTP en el composition root y este se borra.
 */
export function createUnavailableAuthGateway(): AuthGateway {
  const answer = (): Promise<AuthResult> => Promise.resolve(UNAVAILABLE);
  return { signIn: answer, signUp: answer, signInWithProvider: answer };
}
