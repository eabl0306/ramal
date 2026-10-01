/**
 * @ramal/bridge: el puerto `Transport`, el cliente tipado del motor y los tipos
 * del protocolo generados desde `ramal-protocol` con ts-rs. Cero dependencias en
 * tiempo de ejecucion: los transportes concretos viven en cada app.
 */
export {
  BRIDGE_ERROR_KINDS,
  BridgeError,
  isBridgeError,
  isBridgeErrorKind,
  type BridgeErrorDetails,
  type BridgeErrorKind,
} from './bridge-error';
export {
  DEFAULT_TIMEOUT_MS,
  createBridgeClient,
  type AbortSignalLike,
  type BridgeClient,
  type BridgeClientOptions,
  type CallOptions,
  type Scheduler,
} from './client';
export * from './generated';
export type { NotificationListener, Transport, Unsubscribe } from './transport';
