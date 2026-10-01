import { readFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import type { Page } from '@playwright/test';
import type { GraphWindow } from '@ramal/bridge';

import {
  entryAssets,
  parseViteManifest,
} from '../src/extension/infrastructure/webview/vite-manifest';
import {
  createNonce,
  productionCsp,
  renderWebviewPage,
} from '../src/extension/infrastructure/webview/webview-html';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const ORIGIN = 'http://127.0.0.1:4175';
export const PAGE_URL = `${ORIGIN}/graph.html`;

export interface HostScenario {
  readonly locale: 'es' | 'en';
  readonly window: GraphWindow;
  /** Mientras sea verdadero, el host contesta `engine_unavailable`. */
  readonly engineDown: boolean;
}

/** Dos commits con una rama: lo minimo para ver filas, refs y aristas. */
export const TWO_COMMITS: GraphWindow = {
  repoId: 'repo-e2e',
  offset: 0,
  hasMore: false,
  laneCount: 1,
  rows: [
    {
      sha: 'b'.repeat(40),
      shortSha: 'bbbbbbb',
      parents: ['a'.repeat(40)],
      lane: 0,
      edges: [
        {
          parentSha: 'a'.repeat(40),
          parentRow: 1,
          fromLane: 0,
          travelLane: 0,
          toLane: 0,
          kind: 'firstParent',
        },
      ],
      summary: 'Pintar el grafo en VS Code',
      authorName: 'Ana Ruiz',
      authorEmail: 'ana@example.com',
      authoredAt: '2026-09-30T10:00:00+02:00',
      refs: [{ name: 'main', fullName: 'refs/heads/main', kind: 'localBranch', isHead: true }],
      isHead: true,
    },
    {
      sha: 'a'.repeat(40),
      shortSha: 'aaaaaaa',
      parents: [],
      lane: 0,
      edges: [],
      summary: 'Primer commit',
      authorName: 'Luis Gil',
      authorEmail: 'luis@example.com',
      authoredAt: '2026-09-29T09:00:00+02:00',
      refs: [],
      isHead: false,
    },
  ],
};

/**
 * El HTML del webview tal como lo genera el extension host: CSP de produccion,
 * nonce nuevo y los ficheros del manifiesto de Vite. `cspSource` es el origen
 * del servidor de preview, que hace el papel de `webview.cspSource`.
 */
function webviewHtml(locale: string): string {
  const manifestFile = path.join(ROOT, 'dist', 'webview', 'manifest.json');
  const manifest = parseViteManifest(JSON.parse(readFileSync(manifestFile, 'utf8')));
  const assets = entryAssets(manifest, 'src/webview/app/entries/graph.tsx');
  const nonce = createNonce();
  return renderWebviewPage({
    title: 'Ramal Graph',
    lang: locale,
    csp: productionCsp(ORIGIN, nonce),
    nonce,
    styles: assets.styles.map((file) => `${ORIGIN}/${file}`),
    scripts: [`${ORIGIN}/${assets.script}`],
  });
}

/**
 * El extension host de mentira, inyectado antes de cargar la pagina:
 * `acquireVsCodeApi` guarda cada mensaje y contesta como lo haria el host
 * (`init` a `webview/ready`, la ventana a `graph/window`). Corre en el
 * navegador, asi que no puede usar nada de fuera de la funcion.
 */
function installFakeHost(scenario: HostScenario): void {
  const state = { posted: [] as unknown[], engineDown: scenario.engineDown };
  const reply = (data: unknown): void => {
    window.postMessage(data, '*');
  };
  const answer = (request: { id: number; method: string }): void => {
    const response = state.engineDown
      ? {
          jsonrpc: '2.0',
          id: request.id,
          error: { code: -32_000, message: 'engine down', data: { kind: 'engine_unavailable' } },
        }
      : { jsonrpc: '2.0', id: request.id, result: scenario.window };
    reply({ type: 'rpc/response', response });
  };
  Object.assign(window, {
    ramalHost: state,
    acquireVsCodeApi: () => ({
      postMessage: (message: { type: string; request?: { id: number; method: string } }) => {
        state.posted.push(message);
        if (message.type === 'webview/ready') {
          reply({
            type: 'init',
            locale: scenario.locale,
            surface: 'graph',
            repoId: 'repo-e2e',
            protocolVersion: 1,
          });
        }
        if (message.type === 'rpc/request' && message.request !== undefined) {
          answer(message.request);
        }
      },
    }),
  });
}

/** Abre el webview del grafo con el host falso y la CSP de produccion. */
export async function openGraphWebview(page: Page, scenario: HostScenario): Promise<void> {
  await page.addInitScript(installFakeHost, scenario);
  await page.addInitScript(() => {
    const violations: string[] = [];
    document.addEventListener('securitypolicyviolation', (event) => {
      violations.push(`${event.violatedDirective} ${event.blockedURI}`);
    });
    Object.assign(window, { cspViolations: violations });
  });
  await page.route(PAGE_URL, (route) =>
    route.fulfill({ contentType: 'text/html', body: webviewHtml(scenario.locale) }),
  );
  await page.goto(PAGE_URL);
}

/** Lo que el webview mando al host hasta ahora. */
export function postedMessages(page: Page): Promise<unknown[]> {
  return page.evaluate(
    () => (window as unknown as { ramalHost: { posted: unknown[] } }).ramalHost.posted,
  );
}

/** Las violaciones de CSP que vio la pagina. */
export function cspViolations(page: Page): Promise<string[]> {
  return page.evaluate(() => (window as unknown as { cspViolations: string[] }).cspViolations);
}

/** Levanta el motor de mentira: desde ahora contesta bien. */
export async function bringEngineUp(page: Page): Promise<void> {
  await page.evaluate(() => {
    (window as unknown as { ramalHost: { engineDown: boolean } }).ramalHost.engineDown = false;
  });
}
