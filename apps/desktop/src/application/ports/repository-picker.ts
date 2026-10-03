import type { BridgeErrorKind, RepoHandle } from '@git-rudder/bridge';

/**
 * Lo que puede pasar al pedir un repositorio: el usuario elige una carpeta y el
 * motor la abre, el usuario cancela, o algo falla (la carpeta no es un
 * repositorio, falta git, se cae el transporte). El fallo trae el `kind` estable
 * del motor y la interfaz lo traduce con @git-rudder/i18n.
 */
export type PickRepositoryResult =
  | { readonly outcome: 'opened'; readonly repo: RepoHandle }
  | { readonly outcome: 'cancelled' }
  | { readonly outcome: 'failed'; readonly kind: BridgeErrorKind };

/**
 * Puerto del host de escritorio para elegir y abrir un repositorio. El adaptador
 * de Tauri llama a `repo_pick_and_open`, que ensena el dialogo nativo desde Rust:
 * el webview nunca manda una ruta. No lanza nunca: todo resultado es un valor.
 */
export interface RepositoryPicker {
  pickAndOpen(): Promise<PickRepositoryResult>;
}
