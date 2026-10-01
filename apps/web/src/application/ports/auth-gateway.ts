/**
 * Puerto de autenticacion de la web. En el primer corte su unico adaptador
 * contesta que todavia no esta disponible: la API aun no tiene el contexto auth
 * (architecture.md 10). Las pantallas y su validacion ya son las de verdad.
 */

export interface Credentials {
  readonly email: string;
  readonly password: string;
}

export interface Registration {
  readonly name: string;
  readonly email: string;
  readonly password: string;
  /** Opcional en el registro; null si no se escribio. */
  readonly organization: string | null;
}

export type OAuthProvider = 'github' | 'gitlab' | 'google';

export interface Session {
  readonly email: string;
}

/** Motivos de rechazo; la interfaz los traduce con errors.api.<codigo>. */
export type AuthFailure = 'unavailable' | 'network' | 'unknown';

export type AuthResult =
  | { readonly ok: true; readonly session: Session }
  | { readonly ok: false; readonly failure: AuthFailure };

export interface AuthGateway {
  readonly signIn: (credentials: Credentials) => Promise<AuthResult>;
  readonly signUp: (registration: Registration) => Promise<AuthResult>;
  readonly signInWithProvider: (provider: OAuthProvider) => Promise<AuthResult>;
}
