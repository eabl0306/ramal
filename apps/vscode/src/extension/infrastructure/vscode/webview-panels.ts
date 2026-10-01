import { readFileSync } from 'node:fs';

import * as vscode from 'vscode';

import type { Surface } from '../../../shared/messages';
import type { Logger } from '../../application/ports/logger';
import type {
  PanelChannel,
  ShowSurfaceRequest,
  WebviewSurface,
} from '../../application/ports/webview-surface';
import type { WebviewRouter } from '../../application/use-cases/route-webview-message';
import { entryAssets, parseViteManifest, type ViteManifest } from '../webview/vite-manifest';
import {
  createNonce,
  developmentCsp,
  productionCsp,
  reactRefreshPreamble,
  renderWebviewPage,
} from '../webview/webview-html';
import { parseWebviewMessage } from '../webview/webview-message-schema';

export interface WebviewPanelsOptions {
  readonly extensionUri: vscode.Uri;
  /** Idioma de la interfaz, para el `lang` del documento. */
  readonly locale: string;
  readonly logger: Logger;
  readonly router: Pick<WebviewRouter, 'attach' | 'detach' | 'receive'>;
  /** Servidor de Vite; solo llega en `ExtensionMode.Development`. */
  readonly devServer?: string | undefined;
}

/** Entrada de Vite de cada superficie (`build.rolldownOptions.input`). */
function entryOf(surface: Surface): string {
  return `src/webview/app/entries/${surface}.tsx`;
}

/**
 * Los paneles de webview (`ramal.graph` en el primer corte). Cada panel tiene su
 * canal y su superficie, y solo se escucha lo que llega de los paneles que crea
 * este adaptador (architecture.md 6.7, paso 1). Todo mensaje pasa por el
 * esquema de zod antes de llegar al router.
 */
export class VsCodeWebviewPanels implements WebviewSurface, vscode.Disposable {
  readonly #options: WebviewPanelsOptions;
  readonly #webviewRoot: vscode.Uri;
  readonly #panels = new Map<string, vscode.WebviewPanel>();
  #manifest: ViteManifest | undefined;
  #nextId = 0;

  constructor(options: WebviewPanelsOptions) {
    this.#options = options;
    this.#webviewRoot = vscode.Uri.joinPath(options.extensionUri, 'dist', 'webview');
  }

  show(request: ShowSurfaceRequest): void {
    const key = `${request.surface}:${request.repoId}`;
    const existing = this.#panels.get(key);
    if (existing !== undefined) {
      existing.reveal();
      return;
    }
    this.#panels.set(key, this.#create(request, key));
  }

  dispose(): void {
    for (const panel of [...this.#panels.values()]) panel.dispose();
    this.#panels.clear();
  }

  #create(request: ShowSurfaceRequest, key: string): vscode.WebviewPanel {
    const panel = vscode.window.createWebviewPanel(
      `ramal.${request.surface}`,
      request.title,
      vscode.ViewColumn.Active,
      { enableScripts: true, localResourceRoots: [this.#webviewRoot] },
    );
    panel.iconPath = vscode.Uri.joinPath(this.#options.extensionUri, 'media', 'ramal.svg');
    this.#nextId += 1;
    const channel: PanelChannel = {
      id: `panel-${String(this.#nextId)}`,
      surface: request.surface,
      repoId: request.repoId,
      post: (message) => {
        void panel.webview.postMessage(message);
      },
    };
    const { router, logger } = this.#options;
    router.attach(channel);
    const listener = panel.webview.onDidReceiveMessage((raw: unknown) => {
      const parsed = parseWebviewMessage(raw);
      if (parsed.ok) void router.receive(channel.id, parsed.message);
      else logger.warn(`message from ${channel.id} dropped: ${parsed.reason}`);
    });
    panel.onDidDispose(() => {
      listener.dispose();
      router.detach(channel.id);
      this.#panels.delete(key);
    });
    panel.webview.html = this.#html(panel.webview, request);
    return panel;
  }

  #html(webview: vscode.Webview, request: ShowSurfaceRequest): string {
    const nonce = createNonce();
    const common = { title: request.title, lang: this.#options.locale, nonce };
    const { devServer } = this.#options;
    if (devServer !== undefined) {
      return renderWebviewPage({
        ...common,
        csp: developmentCsp(webview.cspSource, nonce, devServer),
        styles: [],
        scripts: [`${devServer}/@vite/client`, `${devServer}/${entryOf(request.surface)}`],
        inlineModule: reactRefreshPreamble(devServer),
      });
    }
    const assets = entryAssets(this.#readManifest(), entryOf(request.surface));
    const toUri = (file: string): string =>
      webview.asWebviewUri(vscode.Uri.joinPath(this.#webviewRoot, file)).toString();
    return renderWebviewPage({
      ...common,
      csp: productionCsp(webview.cspSource, nonce),
      styles: assets.styles.map(toUri),
      scripts: [toUri(assets.script)],
    });
  }

  #readManifest(): ViteManifest {
    if (this.#manifest === undefined) {
      const file = vscode.Uri.joinPath(this.#webviewRoot, 'manifest.json').fsPath;
      this.#manifest = parseViteManifest(JSON.parse(readFileSync(file, 'utf8')));
    }
    return this.#manifest;
  }
}
