import { ENGINE_METHODS, type EngineMethod, HOST_METHODS } from '@ramal/bridge';

import type { Surface } from '../../shared/messages';

/**
 * Metodos del motor que puede pedir cada superficie (architecture.md 6.7). Va
 * tipado contra `EngineMethod`, la lista generada desde `ramal-protocol`: un
 * metodo que desaparece del motor deja de compilar aqui.
 *
 * `repo/open` y `engine/info` no estan en ninguna: solo los llama el host. El
 * primer corte solo monta el grafo, que solo pide su ventana.
 */
export const SURFACE_METHODS: Readonly<Record<Surface, readonly EngineMethod[]>> = {
  graph: ['graph/window'],
  changes: [],
  merge: [],
  settings: [],
};

const ENGINE_METHOD_SET: ReadonlySet<string> = new Set(ENGINE_METHODS);
const HOST_ONLY: ReadonlySet<string> = new Set<string>([...HOST_METHODS, 'engine/info']);

/**
 * Un metodo pasa si existe en el motor, no es de los que solo llama el host y
 * esta en la lista de la superficie. Las tres condiciones a la vez: si alguien
 * anadiera `repo/open` a una superficie por error, la segunda lo sigue parando.
 */
export function isMethodAllowed(surface: Surface, method: string): boolean {
  if (!ENGINE_METHOD_SET.has(method) || HOST_ONLY.has(method)) return false;
  return SURFACE_METHODS[surface].some((allowed) => allowed === method);
}
