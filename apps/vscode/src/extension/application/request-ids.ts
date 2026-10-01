/** Mayor id del protocolo: 2^31 - 1, un `number` seguro y lo que acepta el motor. */
export const MAX_REQUEST_ID = 2_147_483_647;

/** Reparte los ids de las peticiones al motor. */
export interface RequestIds {
  next(): number;
}

/**
 * Un contador por conexion, compartido por el host y por el router de los
 * webviews: asi un id que inventa un webview nunca llega al motor y dos webviews
 * no pueden chocar. Vuelve a 1 al llegar al maximo.
 */
export function createRequestIds(start = 0): RequestIds {
  let last = start;
  return {
    next() {
      last = last >= MAX_REQUEST_ID ? 1 : last + 1;
      return last;
    },
  };
}
