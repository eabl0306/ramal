import { randomBytes } from 'node:crypto';

/**
 * El HTML de un webview y su CSP (architecture.md 6.8). Funciones puras: el
 * adaptador del panel les pasa las URIs ya convertidas con `asWebviewUri`, y
 * los tests de Playwright las usan igual con un servidor local.
 */

/** Un nonce nuevo en cada carga: sin el, ningun script del webview se ejecuta. */
export function createNonce(): string {
  return randomBytes(16).toString('base64');
}

/**
 * La politica de produccion. Sin `'unsafe-inline'` en estilos: ninguna pieza
 * inyecta `<style>` en ejecucion y las props `style` de React van por CSSOM.
 * `connect-src 'none'`: el webview no habla con nadie mas que con el host.
 */
export function productionCsp(cspSource: string, nonce: string): string {
  return [
    "default-src 'none'",
    `script-src 'nonce-${nonce}'`,
    `style-src ${cspSource}`,
    `font-src ${cspSource}`,
    `img-src ${cspSource} data:`,
    "connect-src 'none'",
    "frame-src 'none'",
    "base-uri 'none'",
    "form-action 'none'",
  ].join('; ');
}

/**
 * La politica de desarrollo, solo con `ExtensionMode.Development` y el
 * servidor de Vite en marcha: deja cargar sus modulos, su websocket de recarga
 * y los `<style>` que Vite inyecta en desarrollo.
 */
export function developmentCsp(cspSource: string, nonce: string, devServer: string): string {
  const socket = devServer.replace(/^http/u, 'ws');
  return [
    "default-src 'none'",
    `script-src 'nonce-${nonce}' ${devServer}`,
    `style-src ${cspSource} ${devServer} 'unsafe-inline'`,
    `font-src ${cspSource} ${devServer}`,
    `img-src ${cspSource} ${devServer} data:`,
    `connect-src ${devServer} ${socket}`,
    "frame-src 'none'",
    "base-uri 'none'",
    "form-action 'none'",
  ].join('; ');
}

const LOOPBACK_ORIGIN = /^http:\/\/(localhost|127\.0\.0\.1):\d{2,5}$/u;

/** Solo se acepta un servidor de Vite en la propia maquina. */
export function devServerOrigin(value: string | undefined): string | undefined {
  const trimmed = value?.trim().replace(/\/$/u, '');
  return trimmed !== undefined && LOOPBACK_ORIGIN.test(trimmed) ? trimmed : undefined;
}

export interface WebviewPage {
  readonly title: string;
  readonly lang: string;
  readonly csp: string;
  readonly nonce: string;
  readonly styles: readonly string[];
  readonly scripts: readonly string[];
  /** Modulo en linea (el preambulo de React Refresh en desarrollo). */
  readonly inlineModule?: string;
}

const ESCAPES: Readonly<Record<string, string>> = {
  '&': '&amp;',
  '<': '&lt;',
  '>': '&gt;',
  '"': '&quot;',
  "'": '&#39;',
};

/** Escapa texto para HTML, tambien dentro de atributos. */
export function escapeHtml(text: string): string {
  return text.replace(/[&<>"']/gu, (character) => ESCAPES[character] ?? character);
}

/** El documento completo del webview. */
export function renderWebviewPage(page: WebviewPage): string {
  const nonce = escapeHtml(page.nonce);
  const styles = page.styles.map((href) => `<link rel="stylesheet" href="${escapeHtml(href)}">`);
  const inline =
    page.inlineModule === undefined
      ? []
      : [`<script type="module" nonce="${nonce}">${page.inlineModule}</script>`];
  const scripts = page.scripts.map(
    (src) => `<script type="module" nonce="${nonce}" src="${escapeHtml(src)}"></script>`,
  );
  return [
    '<!doctype html>',
    `<html lang="${escapeHtml(page.lang)}">`,
    '<head>',
    '<meta charset="utf-8">',
    `<meta http-equiv="Content-Security-Policy" content="${escapeHtml(page.csp)}">`,
    '<meta name="viewport" content="width=device-width, initial-scale=1">',
    `<title>${escapeHtml(page.title)}</title>`,
    ...styles,
    '</head>',
    '<body>',
    '<div id="root"></div>',
    ...inline,
    ...scripts,
    '</body>',
    '</html>',
  ].join('\n');
}

/** El preambulo que @vitejs/plugin-react necesita antes de cargar un modulo en desarrollo. */
export function reactRefreshPreamble(devServer: string): string {
  return [
    `import RefreshRuntime from '${devServer}/@react-refresh';`,
    'RefreshRuntime.injectIntoGlobalHook(window);',
    'window.$RefreshReg$ = () => {};',
    'window.$RefreshSig$ = () => (type) => type;',
    'window.__vite_plugin_react_preamble_installed__ = true;',
  ].join('\n');
}
