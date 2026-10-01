// Generado por crates/protocol/tests/export_bindings.rs (just protocol-gen). No editar a mano.
import type { EngineInfo } from "./EngineInfo";
import type { EngineInfoParams } from "./EngineInfoParams";
import type { GraphWindow } from "./GraphWindow";
import type { GraphWindowParams } from "./GraphWindowParams";
import type { OpenRepoParams } from "./OpenRepoParams";
import type { RepoHandle } from "./RepoHandle";

/** Version del protocolo del motor. */
export const PROTOCOL_VERSION = 2;

/** Cada metodo del motor con sus parametros y su resultado. */
export type EngineMethods = {
  "engine/info": { params: EngineInfoParams; result: EngineInfo };
  "repo/open": { params: OpenRepoParams; result: RepoHandle };
  "graph/window": { params: GraphWindowParams; result: GraphWindow };
};

/** Nombre de un metodo del motor. */
export type EngineMethod = keyof EngineMethods;

/** Todos los metodos: la lista blanca del extension host. */
export const ENGINE_METHODS = ["engine/info", "repo/open", "graph/window"] as const satisfies readonly EngineMethod[];

/** Metodos que solo puede llamar un host: nunca un webview. */
export const HOST_METHODS = ["repo/open"] as const satisfies readonly EngineMethod[];
