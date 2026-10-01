import type { BridgeErrorKind, RepoHandle } from '@ramal/bridge';

import type { PickRepositoryResult } from '../ports/repository-picker';

/** Un repositorio abierto en su pestana. */
export interface RepoTab {
  readonly repoId: string;
  /** Raiz del arbol de trabajo, solo para ensenarla. */
  readonly root: string;
  /** Nombre de la pestana: la ultima carpeta de la raiz. */
  readonly label: string;
}

/**
 * Las pestanas de la ventana y lo que pasa en Inicio. `active` es el `repoId` de
 * la pestana visible, o `null` cuando se ve Inicio.
 */
export interface WorkspaceState {
  readonly tabs: readonly RepoTab[];
  readonly active: string | null;
  /** Se esta esperando al dialogo de carpetas. */
  readonly picking: boolean;
  /** Por que fallo la ultima apertura, hasta que se intenta otra. */
  readonly openError: BridgeErrorKind | null;
}

export type WorkspaceAction =
  | { readonly type: 'pick-started' }
  | { readonly type: 'pick-finished'; readonly result: PickRepositoryResult }
  | { readonly type: 'tab-activated'; readonly repoId: string | null }
  | { readonly type: 'tab-closed'; readonly repoId: string };

export const INITIAL_WORKSPACE: WorkspaceState = {
  tabs: [],
  active: null,
  picking: false,
  openError: null,
};

/**
 * Orquestacion de la pantalla: abrir un repositorio abre su pestana (o vuelve a la
 * que ya tenia), cerrar la pestana visible pasa a la ultima que queda y, si no
 * queda ninguna, a Inicio, como en el diseno.
 */
export function workspaceReducer(state: WorkspaceState, action: WorkspaceAction): WorkspaceState {
  switch (action.type) {
    case 'pick-started':
      return { ...state, picking: true, openError: null };
    case 'pick-finished':
      return finishPick({ ...state, picking: false }, action.result);
    case 'tab-activated':
      return { ...state, active: activeOrHome(state, action.repoId) };
    case 'tab-closed':
      return closeTab(state, action.repoId);
  }
}

/** La pestana visible, o `null` en Inicio. */
export function activeTab(state: WorkspaceState): RepoTab | null {
  return state.tabs.find((tab) => tab.repoId === state.active) ?? null;
}

/**
 * Nombre corto de un repositorio: la ultima carpeta de su raiz, con barras de Unix
 * o de Windows y sin la barra final. Si no queda nada (la raiz del disco), la
 * raiz entera.
 */
export function repoLabel(root: string): string {
  const parts = root.split(/[\\/]/u).filter((part) => part !== '');
  return parts.at(-1) ?? root;
}

function finishPick(state: WorkspaceState, result: PickRepositoryResult): WorkspaceState {
  switch (result.outcome) {
    case 'opened':
      return openTab(state, result.repo);
    case 'cancelled':
      return state;
    case 'failed':
      return { ...state, openError: result.kind };
  }
}

function openTab(state: WorkspaceState, repo: RepoHandle): WorkspaceState {
  const known = state.tabs.some((tab) => tab.repoId === repo.repoId);
  const tabs = known
    ? state.tabs
    : [...state.tabs, { repoId: repo.repoId, root: repo.root, label: repoLabel(repo.root) }];
  return { ...state, tabs, active: repo.repoId };
}

function closeTab(state: WorkspaceState, repoId: string): WorkspaceState {
  const tabs = state.tabs.filter((tab) => tab.repoId !== repoId);
  if (state.active !== repoId) return { ...state, tabs };
  return { ...state, tabs, active: tabs.at(-1)?.repoId ?? null };
}

function activeOrHome(state: WorkspaceState, repoId: string | null): string | null {
  if (repoId === null) return null;
  return state.tabs.some((tab) => tab.repoId === repoId) ? repoId : state.active;
}
