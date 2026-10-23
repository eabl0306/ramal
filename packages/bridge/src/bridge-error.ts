import type { ErrorKind, RpcError } from './generated';

/**
 * Tipo de error que ve la interfaz: el `ErrorKind` cerrado del motor mas los tres
 * que anaden el puente y el extension host. La interfaz traduce cada uno con
 * `@git-rudder/i18n`; el `message` es solo para el log.
 */
export type BridgeErrorKind = ErrorKind | 'transport' | 'timeout' | 'engine_unavailable';

/**
 * Todos los tipos, como registro: si el motor anade un `ErrorKind`, esto deja de
 * compilar hasta que se anade aqui.
 */
const KINDS: Readonly<Record<BridgeErrorKind, true>> = {
  invalid_params: true,
  repo_not_open: true,
  not_a_repository: true,
  git_not_found: true,
  git_version_unsupported: true,
  git_failed: true,
  stale_window: true,
  internal: true,
  transport: true,
  timeout: true,
  engine_unavailable: true,
};

/** Codigo JSON-RPC de parametros invalidos. */
const INVALID_PARAMS_CODE = -32_602;

/** Lista de todos los tipos de error. */
export const BRIDGE_ERROR_KINDS = Object.keys(KINDS) as readonly BridgeErrorKind[];

/** Comprueba que un valor recibido es un tipo de error conocido. */
export function isBridgeErrorKind(value: unknown): value is BridgeErrorKind {
  return typeof value === 'string' && Object.hasOwn(KINDS, value);
}

/** Datos opcionales de un `BridgeError`. */
export interface BridgeErrorDetails {
  /** Codigo JSON-RPC, si el error vino del motor. */
  readonly code?: number;
  /** `error.data` tal como llego. */
  readonly data?: unknown;
  /** Error original, si lo hubo. */
  readonly cause?: unknown;
}

/** Fallo de una llamada al motor. */
export class BridgeError extends Error {
  override readonly name = 'BridgeError';
  readonly kind: BridgeErrorKind;
  readonly code: number | undefined;
  readonly data: unknown;

  constructor(kind: BridgeErrorKind, message: string, details: BridgeErrorDetails = {}) {
    super(message, { cause: details.cause });
    this.kind = kind;
    this.code = details.code;
    this.data = details.data;
  }
}

/** Distingue un `BridgeError` de cualquier otro valor lanzado. */
export function isBridgeError(value: unknown): value is BridgeError {
  return value instanceof BridgeError;
}

/**
 * Error de una respuesta del motor. Se lee `data.kind` sin fiarse del tipo: el
 * extension host puede mandar `engine_unavailable`, que no es del motor. Sin un
 * `kind` conocido, se deduce del codigo.
 */
export function fromRpcError(error: RpcError): BridgeError {
  const raw: unknown = (error.data as { readonly kind?: unknown } | undefined)?.kind;
  const kind = isBridgeErrorKind(raw) ? raw : kindFromCode(error.code);
  return new BridgeError(kind, error.message, { code: error.code, data: error.data });
}

function kindFromCode(code: number): BridgeErrorKind {
  return code === INVALID_PARAMS_CODE ? 'invalid_params' : 'internal';
}
