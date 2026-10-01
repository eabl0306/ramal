import type { ErrorKind, RpcError } from '@ramal/bridge';
import type { PlainKey } from '@ramal/i18n';

import type { HostErrorKind } from '../../shared/messages';

/**
 * El motor no esta: no arranco, se cayo a mitad de una peticion, esta esperando
 * para reiniciarse o el host dejo de intentarlo. Las peticiones pendientes
 * fallan con esto en vez de quedarse colgadas (ADR 0010).
 */
export class EngineUnavailableError extends Error {
  override readonly name = 'EngineUnavailableError';
}

/**
 * Error que devolvio el motor a una peticion del propio host. Guarda el error
 * tal cual para poder reenviarlo a un webview con su tipo.
 */
export class EngineCallError extends Error {
  override readonly name = 'EngineCallError';
  readonly rpc: RpcError;
  readonly kind: ErrorKind | undefined;

  constructor(error: RpcError) {
    super(error.message);
    this.rpc = error;
    this.kind = error.data?.kind;
  }
}

/** Lo que el host ensena de un error: el `kind` del motor o uno suyo. */
export type HostVisibleErrorKind = ErrorKind | HostErrorKind;

/**
 * El texto de cada tipo de error. Un Record sobre todos: un `ErrorKind` nuevo en
 * el motor no compila hasta que tiene su clave aqui y en @ramal/i18n.
 */
const ERROR_KEYS: Readonly<Record<HostVisibleErrorKind, PlainKey>> = {
  invalid_params: 'errors.engine.invalid_params',
  repo_not_open: 'errors.engine.repo_not_open',
  not_a_repository: 'errors.engine.not_a_repository',
  git_not_found: 'errors.engine.git_not_found',
  git_version_unsupported: 'errors.engine.git_version_unsupported',
  git_failed: 'errors.engine.git_failed',
  internal: 'errors.engine.internal',
  engine_unavailable: 'errors.engine.engine_unavailable',
};

/** Tipo de un error cualquiera; lo desconocido es interno. */
export function errorKindOf(error: unknown): HostVisibleErrorKind {
  if (error instanceof EngineUnavailableError) return 'engine_unavailable';
  if (error instanceof EngineCallError) return error.kind ?? 'internal';
  return 'internal';
}

/** Clave de i18n con la que se ensena un error. */
export function errorKey(error: unknown): PlainKey {
  return ERROR_KEYS[errorKindOf(error)];
}

/** Descripcion corta de un error para el log, en ingles. */
export function describeError(error: unknown): string {
  return error instanceof Error ? `${error.name}: ${error.message}` : String(error);
}
