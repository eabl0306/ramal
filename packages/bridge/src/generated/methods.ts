// Generado por crates/protocol/tests/export_bindings.rs (just protocol-gen). No editar a mano.
import type { Blame } from "./Blame";
import type { BlameParams } from "./BlameParams";
import type { CommitDetail } from "./CommitDetail";
import type { CommitDetailParams } from "./CommitDetailParams";
import type { CompareParams } from "./CompareParams";
import type { Comparison } from "./Comparison";
import type { ConfigureGitParams } from "./ConfigureGitParams";
import type { ConflictFile } from "./ConflictFile";
import type { DiffFileParams } from "./DiffFileParams";
import type { EngineInfo } from "./EngineInfo";
import type { EngineInfoParams } from "./EngineInfoParams";
import type { FileDiff } from "./FileDiff";
import type { FileHistoryPage } from "./FileHistoryPage";
import type { FileHistoryParams } from "./FileHistoryParams";
import type { GitInfo } from "./GitInfo";
import type { GraphSearchParams } from "./GraphSearchParams";
import type { GraphSearchResult } from "./GraphSearchResult";
import type { GraphWindow } from "./GraphWindow";
import type { GraphWindowParams } from "./GraphWindowParams";
import type { OpenRepoParams } from "./OpenRepoParams";
import type { RefListing } from "./RefListing";
import type { RepoChangedParams } from "./RepoChangedParams";
import type { RepoHandle } from "./RepoHandle";
import type { RepoParams } from "./RepoParams";
import type { RepoPathParams } from "./RepoPathParams";
import type { RepoSummary } from "./RepoSummary";
import type { RepoSummaryParams } from "./RepoSummaryParams";
import type { RepoTrusted } from "./RepoTrusted";
import type { ScanReposParams } from "./ScanReposParams";
import type { ScanReposResult } from "./ScanReposResult";
import type { StashContents } from "./StashContents";
import type { StashList } from "./StashList";
import type { StashShowParams } from "./StashShowParams";
import type { SubmoduleList } from "./SubmoduleList";
import type { WorkingTreeStatus } from "./WorkingTreeStatus";

/** Version del protocolo del motor. */
export const PROTOCOL_VERSION = 3;

/** Cada metodo del motor con sus parametros y su resultado. */
export type EngineMethods = {
  "engine/info": { params: EngineInfoParams; result: EngineInfo };
  "git/configure": { params: ConfigureGitParams; result: GitInfo };
  "repo/open": { params: OpenRepoParams; result: RepoHandle };
  "repo/trust": { params: RepoParams; result: RepoTrusted };
  "repos/summary": { params: RepoSummaryParams; result: RepoSummary };
  "repos/scan": { params: ScanReposParams; result: ScanReposResult };
  "submodule/open": { params: RepoPathParams; result: RepoHandle };
  "graph/window": { params: GraphWindowParams; result: GraphWindow };
  "graph/search": { params: GraphSearchParams; result: GraphSearchResult };
  "status/read": { params: RepoParams; result: WorkingTreeStatus };
  "diff/file": { params: DiffFileParams; result: FileDiff };
  "commit/detail": { params: CommitDetailParams; result: CommitDetail };
  "refs/list": { params: RepoParams; result: RefListing };
  "stash/list": { params: RepoParams; result: StashList };
  "stash/show": { params: StashShowParams; result: StashContents };
  "submodule/list": { params: RepoParams; result: SubmoduleList };
  "history/file": { params: FileHistoryParams; result: FileHistoryPage };
  "blame/file": { params: BlameParams; result: Blame };
  "compare/refs": { params: CompareParams; result: Comparison };
  "conflict/read": { params: RepoPathParams; result: ConflictFile };
};

/** Nombre de un metodo del motor. */
export type EngineMethod = keyof EngineMethods;

/** Todos los metodos: la lista blanca del extension host. */
export const ENGINE_METHODS = ["engine/info", "git/configure", "repo/open", "repo/trust", "repos/summary", "repos/scan", "submodule/open", "graph/window", "graph/search", "status/read", "diff/file", "commit/detail", "refs/list", "stash/list", "stash/show", "submodule/list", "history/file", "blame/file", "compare/refs", "conflict/read"] as const satisfies readonly EngineMethod[];

/** Metodos que solo puede llamar un host: nunca un webview. */
export const HOST_METHODS = ["git/configure", "repo/open", "repo/trust", "repos/summary", "repos/scan", "submodule/open"] as const satisfies readonly EngineMethod[];

/** Cada notificacion del motor hacia la interfaz con sus parametros. */
export type EngineNotifications = {
  "repo/changed": RepoChangedParams;
};

/** Nombre de una notificacion del motor. */
export type EngineNotificationName = keyof EngineNotifications;

/** Todas las notificaciones que manda el motor. */
export const ENGINE_NOTIFICATIONS = ["repo/changed"] as const satisfies readonly EngineNotificationName[];
