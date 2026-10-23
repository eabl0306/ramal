import { type BridgeErrorKind, isBridgeError } from '@git-rudder/bridge';
import type { PlainKey } from '@git-rudder/i18n';

/**
 * El motor no devuelve textos traducidos: devuelve un `kind`, y la interfaz lo
 * traduce (architecture.md 7.5). Es un Record sobre todos los tipos, asi que un
 * `ErrorKind` nuevo en el motor, o uno nuevo del bridge, no compila hasta que
 * tiene su texto aqui y en @git-rudder/i18n.
 */
export const ENGINE_ERROR_KEYS: Readonly<Record<BridgeErrorKind, PlainKey>> = {
  invalid_params: 'errors.engine.invalid_params',
  repo_not_open: 'errors.engine.repo_not_open',
  not_a_repository: 'errors.engine.not_a_repository',
  git_not_found: 'errors.engine.git_not_found',
  git_version_unsupported: 'errors.engine.git_version_unsupported',
  git_failed: 'errors.engine.git_failed',
  // useGraphWindow no lo ensena nunca: vuelve a pedir desde la fila 0. Si
  // llegara a la pantalla seria un fallo de la vista, asi que cuenta como interno.
  stale_window: 'errors.engine.internal',
  internal: 'errors.engine.internal',
  transport: 'errors.engine.transport',
  timeout: 'errors.engine.timeout',
  engine_unavailable: 'errors.engine.engine_unavailable',
};

/**
 * Clave del texto de un error cualquiera. Lo que no es un `BridgeError` (un
 * fallo de programacion en la vista) se ensena como error interno: el detalle
 * en ingles es para el log, no para la persona.
 */
export function engineErrorKey(error: unknown): PlainKey {
  return isBridgeError(error) ? ENGINE_ERROR_KEYS[error.kind] : ENGINE_ERROR_KEYS.internal;
}
