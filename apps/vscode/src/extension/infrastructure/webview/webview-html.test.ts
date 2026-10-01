import { describe, expect, it } from 'vitest';

import { entryAssets, parseViteManifest } from './vite-manifest';
import {
  createNonce,
  developmentCsp,
  devServerOrigin,
  escapeHtml,
  productionCsp,
  reactRefreshPreamble,
  renderWebviewPage,
} from './webview-html';

const SOURCE = 'https://file+.vscode-resource.vscode-cdn.net';

describe('CSP de los webviews', () => {
  it('es exactamente la de produccion de architecture.md 6.8', () => {
    expect(productionCsp(SOURCE, 'abc')).toBe(
      [
        "default-src 'none'",
        "script-src 'nonce-abc'",
        `style-src ${SOURCE}`,
        `font-src ${SOURCE}`,
        `img-src ${SOURCE} data:`,
        "connect-src 'none'",
        "frame-src 'none'",
        "base-uri 'none'",
        "form-action 'none'",
      ].join('; '),
    );
  });

  it('la de desarrollo solo abre el servidor de Vite y su websocket', () => {
    const csp = developmentCsp(SOURCE, 'abc', 'http://127.0.0.1:5175');
    expect(csp).toContain("script-src 'nonce-abc' http://127.0.0.1:5175");
    expect(csp).toContain('connect-src http://127.0.0.1:5175 ws://127.0.0.1:5175');
    expect(csp).toContain("default-src 'none'");
  });

  it('solo acepta un servidor de desarrollo en la propia maquina', () => {
    expect(devServerOrigin('http://127.0.0.1:5175/')).toBe('http://127.0.0.1:5175');
    expect(devServerOrigin('http://localhost:5175')).toBe('http://localhost:5175');
    expect(devServerOrigin('https://evil.example:5175')).toBeUndefined();
    expect(devServerOrigin('http://localhost.evil.example:5175')).toBeUndefined();
    expect(devServerOrigin(undefined)).toBeUndefined();
  });

  it('cada nonce es nuevo y vale como base64', () => {
    const first = createNonce();
    expect(first).toMatch(/^[A-Za-z0-9+/]{22}==$/u);
    expect(createNonce()).not.toBe(first);
  });
});

describe('documento del webview', () => {
  it('lleva la CSP, las hojas, el modulo con su nonce y el contenedor', () => {
    const html = renderWebviewPage({
      title: 'Grafo de Ramal',
      lang: 'es',
      csp: productionCsp(SOURCE, 'n0nce'),
      nonce: 'n0nce',
      styles: [`${SOURCE}/dist/webview/assets/graph.css`],
      scripts: [`${SOURCE}/dist/webview/assets/graph.js`],
    });
    expect(html).toContain('<html lang="es">');
    expect(html).toContain(`content="default-src &#39;none&#39;; script-src &#39;nonce-n0nce&#39;`);
    expect(html).toContain(
      `<link rel="stylesheet" href="${SOURCE}/dist/webview/assets/graph.css">`,
    );
    expect(html).toContain(
      `<script type="module" nonce="n0nce" src="${SOURCE}/dist/webview/assets/graph.js"></script>`,
    );
    expect(html).toContain('<div id="root"></div>');
    expect(html).not.toContain('<style');
  });

  it('escapa el titulo y las URLs', () => {
    expect(escapeHtml(`<script>"x" & 'y'</script>`)).toBe(
      '&lt;script&gt;&quot;x&quot; &amp; &#39;y&#39;&lt;/script&gt;',
    );
    const html = renderWebviewPage({
      title: '<b>',
      lang: 'en',
      csp: '',
      nonce: 'n',
      styles: [],
      scripts: [],
    });
    expect(html).toContain('<title>&lt;b&gt;</title>');
  });

  it('en desarrollo mete el preambulo de React Refresh con el mismo nonce', () => {
    const preamble = reactRefreshPreamble('http://127.0.0.1:5175');
    const html = renderWebviewPage({
      title: 't',
      lang: 'en',
      csp: '',
      nonce: 'n',
      styles: [],
      scripts: [],
      inlineModule: preamble,
    });
    expect(preamble).toContain("from 'http://127.0.0.1:5175/@react-refresh'");
    expect(html).toContain(`<script type="module" nonce="n">${preamble}</script>`);
  });
});

describe('manifiesto de Vite', () => {
  const manifest = parseViteManifest({
    'src/webview/app/entries/graph.tsx': {
      file: 'assets/graph.js',
      isEntry: true,
      css: ['assets/graph.css'],
      imports: ['_shared.js', '_other.js'],
    },
    '_shared.js': { file: 'assets/shared.js', css: ['assets/shared.css'], imports: ['_other.js'] },
    '_other.js': { file: 'assets/other.js', css: ['assets/shared.css'], imports: ['_shared.js'] },
  });

  it('da el modulo de la entrada y todos sus CSS sin repetir, dependencias primero', () => {
    expect(entryAssets(manifest, 'src/webview/app/entries/graph.tsx')).toEqual({
      script: 'assets/graph.js',
      styles: ['assets/shared.css', 'assets/graph.css'],
    });
  });

  it('falla con una entrada que no existe o un manifiesto que no es de Vite', () => {
    expect(() => entryAssets(manifest, 'src/webview/app/entries/merge.tsx')).toThrow(
      'the webview build has no entry src/webview/app/entries/merge.tsx',
    );
    expect(() => parseViteManifest({ entry: { css: [] } })).toThrow();
  });
});
