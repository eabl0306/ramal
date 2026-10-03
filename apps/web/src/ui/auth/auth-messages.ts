import type { AuthFieldError } from '@git-rudder/domain';
import type { PlainKey } from '@git-rudder/i18n';

import type { AuthFailure } from '../../application/ports/auth-gateway';

/**
 * De codigo a texto. @git-rudder/domain y los adaptadores devuelven codigos estables;
 * el texto depende del idioma y lo pone la interfaz. Al ser Record, un codigo
 * nuevo sin traduccion no compila.
 */

type FieldErrorId = `${AuthFieldError['field']}:${AuthFieldError['code']}`;

const FIELD_MESSAGES: Readonly<Partial<Record<FieldErrorId, PlainKey>>> = {
  'name:required': 'web.auth.error.name',
  'email:invalid': 'web.auth.error.email',
  'password:required': 'web.auth.error.passwordRequired',
  'password:too-short': 'web.auth.error.passwordTooShort',
  'terms:required': 'web.auth.error.terms',
};

export const FAILURE_MESSAGES: Readonly<Record<AuthFailure, PlainKey>> = {
  unavailable: 'errors.api.unavailable',
  network: 'errors.api.network',
  unknown: 'errors.api.unknown',
};

/**
 * Se ensena un solo error, el del primer campo que falla en el orden de la
 * pantalla, como hace el diseno con su caja de error.
 */
export function firstFieldMessage(errors: readonly AuthFieldError[]): PlainKey {
  const first = errors[0];
  if (first === undefined) return FAILURE_MESSAGES.unknown;
  return FIELD_MESSAGES[`${first.field}:${first.code}`] ?? FAILURE_MESSAGES.unknown;
}
