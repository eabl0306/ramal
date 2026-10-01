import react from '@vitejs/plugin-react';
import { defineConfig } from 'vite';

/** Puerto del servidor de Vite para los webviews en desarrollo (`pnpm dev`). */
const DEV_PORT = 5175;

/**
 * Build multipagina de los webviews en `dist/webview`: una entrada de JS por
 * superficie (el primer corte solo tiene `graph`). No hay HTML de Vite: el
 * extension host escribe el documento con su CSP y su nonce a partir de
 * `manifest.json` (architecture.md 6.8).
 *
 * `base: './'` deja las URLs relativas al fichero que las pide, que es lo que
 * funciona detras de `asWebviewUri` (las fuentes de Codicons cuelgan del CSS).
 */
export default defineConfig({
  plugins: [react()],
  base: './',
  build: {
    outDir: 'dist/webview',
    emptyOutDir: true,
    manifest: 'manifest.json',
    sourcemap: true,
    // Sin el polyfill de modulepreload: no hace falta en el Chromium de VS Code
    // y es codigo que el CSP tendria que dejar pasar.
    modulePreload: { polyfill: false },
    rolldownOptions: {
      input: { graph: 'src/webview/app/entries/graph.tsx' },
    },
  },
  server: {
    host: '127.0.0.1',
    port: DEV_PORT,
    strictPort: true,
    // El webview carga desde su propio origen (vscode-webview://...).
    cors: true,
    hmr: { host: '127.0.0.1', port: DEV_PORT },
  },
});
