import type { HostToWebviewMessage, Surface } from '../../../shared/messages';

/**
 * El canal de un panel abierto: su superficie, el repositorio que ensena y como
 * mandarle un mensaje. Lo crea el adaptador del panel y lo registra en el router.
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
  /** Titulo de la pestana, ya traducido. */
  readonly title: string;
}

/** Paneles de webview de VS Code. */
export interface WebviewSurface {
  /** Revela el panel de esa superficie y repositorio si existe; si no, lo crea. */
  show(request: ShowSurfaceRequest): void;
}
