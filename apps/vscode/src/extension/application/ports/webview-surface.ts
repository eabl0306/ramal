import type { HostToWebviewMessage, Surface } from '../../../shared/messages';

/**
 * El canal de un panel abierto: su superficie, el repositorio que ensena y como
 * mandarle un mensaje. Lo crea el adaptador del panel y lo registra en el router.
 * `repoId` es el actual: cambia si el host vuelve a abrir el repositorio tras
 * reiniciar el motor y este le da otro id.
 */
export interface PanelChannel {
  readonly id: string;
  readonly surface: Surface;
  readonly repoId: string;
  post(message: HostToWebviewMessage): void;
}

/** Que panel ensenar. */
export interface ShowSurfaceRequest {
  readonly surface: Surface;
  readonly repoId: string;
  /**
   * Raiz del repositorio que devolvio `repo/open`. Identifica el panel (no el
   * `repoId`, que es del proceso del motor) y sirve para volver a abrirlo tras
   * un reinicio. Se queda en el host: nunca llega al webview.
   */
  readonly root: string;
  /** Titulo de la pestana, ya traducido. */
  readonly title: string;
}

/** Paneles de webview de VS Code. */
export interface WebviewSurface {
  /** Revela el panel de esa superficie y raiz si existe; si no, lo crea. */
  show(request: ShowSurfaceRequest): void;
  /** Raices de los repositorios con algun panel abierto, sin repetir. */
  openRoots(): readonly string[];
  /**
   * El motor dio `repoId` a `root` al volver a abrirlo. Si cambio, los paneles de
   * esa raiz lo adoptan y se recargan para recibir un `init` nuevo.
   */
  rebind(root: string, repoId: string): void;
  /** Cierra los paneles de una raiz que ya no se puede abrir. */
  close(root: string): void;
}
