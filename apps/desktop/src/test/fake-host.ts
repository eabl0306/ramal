import type {
  GraphRow,
  GraphWindowParams,
  RepoHandle,
  RpcRequest,
  RpcResponse,
} from '@git-rudder/bridge';

import type { Invoke } from '../infrastructure/tauri/invoke';

/** Fila del grafo con valores razonables; cada test cambia lo que le importa. */
export function graphRow(index: number, summary: string): GraphRow {
  const sha = String(index).padStart(40, 'a');
  return {
    sha,
    shortSha: sha.slice(0, 7),
    parents: [],
    lane: 0,
    edges: [],
    summary,
    authorName: 'Ana Ruiz',
    authorEmail: 'ana@example.com',
    authoredAt: '2026-09-30T10:00:00+02:00',
    refs: [],
    isHead: index === 0,
  };
}

export const GIT_RUDDER_REPO: RepoHandle = {
  repoId: 'repo-1',
  root: '/home/ana/code/git-rudder',
  headRef: 'main',
  headSha: 'a'.repeat(40),
  isDetached: false,
  isEmpty: false,
};

/** Lo que contesta `repo_pick_and_open`: el repo, `null` (cancelar) o un rechazo. */
export type PickAnswer = RepoHandle | null | { readonly reject: unknown };

/**
 * El host de Tauri visto desde el webview: un `invoke` que contesta como los dos
 * comandos de src-tauri. Los tests montan encima los adaptadores reales
 * (TauriTransport y el selector), asi cubren tambien el sobre y los errores.
 */
export class FakeTauriHost {
  readonly requests: RpcRequest[] = [];
  pick: PickAnswer = GIT_RUDDER_REPO;
  readonly #rows: readonly GraphRow[];

  constructor(rows: readonly GraphRow[]) {
    this.#rows = rows;
  }

  readonly invoke: Invoke = async <T>(command: string, args?: Record<string, unknown>) => {
    await Promise.resolve();
    if (command === 'repo_pick_and_open') return this.answerPick() as T;
    if (command === 'rpc') return this.answerRpc(args?.['request'] as RpcRequest) as T;
    throw new Error(`unknown command ${command}`);
  };

  private answerPick(): RepoHandle | null {
    const answer = this.pick;
    if (answer !== null && 'reject' in answer) throw answer.reject;
    return answer;
  }

  private answerRpc(request: RpcRequest): RpcResponse | null {
    this.requests.push(request);
    if (request.id === undefined) return null;
    const { repoId, offset, limit } = request.params as GraphWindowParams;
    const rows = this.#rows.slice(offset, offset + limit);
    return {
      jsonrpc: '2.0',
      id: request.id,
      result: { repoId, offset, rows, hasMore: offset + limit < this.#rows.length, laneCount: 1 },
    };
  }
}
