import type { HostMethod, HostMethods } from '../../../shared/messages';

/**
 * Peticiones nativas de los webviews (`host/request`): las resuelve el host con
 * la API de VS Code, sin pasar por el motor. Los parametros llegan ya validados
 * por el esquema de su metodo.
 */
export interface HostRequests {
  handle<M extends HostMethod>(
    method: M,
    params: HostMethods[M]['params'],
  ): Promise<HostMethods[M]['result']>;
}
