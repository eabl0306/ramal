/**
 * Validacion de los formularios de entrar y de crear cuenta de la web.
 *
 * Es la primera red, para responder al instante: la API vuelve a validar y es la
 * que manda. Devuelve codigos, no textos, porque el texto depende del idioma y
 * lo pone la interfaz con @git-rudder/i18n.
 */

/** La regla del prototipo: algo, una arroba, algo, un punto y algo, sin espacios. */
const EMAIL_PATTERN = /^\S+@\S+\.\S+$/u;

/** Minimo de la contrasena al crear cuenta. Al entrar no se aplica (gaps B-53). */
export const MIN_PASSWORD_LENGTH = 8;

export interface SignupForm {
  readonly name: string;
  readonly email: string;
  readonly password: string;
  readonly termsAccepted: boolean;
}

export interface LoginForm {
  readonly email: string;
  readonly password: string;
}

export type AuthFieldError =
  | { readonly field: 'name'; readonly code: 'required' }
  | { readonly field: 'email'; readonly code: 'invalid' }
  | { readonly field: 'password'; readonly code: 'required' | 'too-short' }
  | { readonly field: 'terms'; readonly code: 'required' };

/**
 * Resultado de validar. Los errores van en el orden de los campos en pantalla:
 * quien solo quiera ensenar uno (como el prototipo) toma el primero, y quien
 * quiera marcar cada campo (gaps C-03) los tiene todos.
 */
export type AuthValidation =
  { readonly ok: true } | { readonly ok: false; readonly errors: readonly AuthFieldError[] };

export type PasswordHint = 'too-short' | 'ok';

/** El email se recorta antes de mirar el patron: pegar con un espacio no es un error. */
export function isValidEmail(email: string): boolean {
  return EMAIL_PATTERN.test(email.trim());
}

/** La pista en vivo del registro: "Minimo 8 caracteres" o "Contrasena valida". */
export function passwordHint(password: string): PasswordHint {
  return password.length >= MIN_PASSWORD_LENGTH ? 'ok' : 'too-short';
}

function toValidation(errors: readonly AuthFieldError[]): AuthValidation {
  return errors.length === 0 ? { ok: true } : { ok: false, errors };
}

/** Crear cuenta: nombre, email, contrasena de 8 o mas y terminos aceptados, en ese orden. */
export function validateSignup(form: SignupForm): AuthValidation {
  const errors: AuthFieldError[] = [];
  if (form.name.trim() === '') errors.push({ field: 'name', code: 'required' });
  if (!isValidEmail(form.email)) errors.push({ field: 'email', code: 'invalid' });
  if (passwordHint(form.password) === 'too-short') {
    errors.push({ field: 'password', code: 'too-short' });
  }
  if (!form.termsAccepted) errors.push({ field: 'terms', code: 'required' });
  return toValidation(errors);
}

/**
 * Entrar: email valido y contrasena no vacia. El minimo de longitud no se pide
 * aqui, porque una cuenta antigua o creada con otra regla tiene que poder entrar.
 */
export function validateLogin(form: LoginForm): AuthValidation {
  const errors: AuthFieldError[] = [];
  if (!isValidEmail(form.email)) errors.push({ field: 'email', code: 'invalid' });
  if (form.password === '') errors.push({ field: 'password', code: 'required' });
  return toValidation(errors);
}
