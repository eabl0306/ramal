import { readFileSync } from 'node:fs';

import react from '@vitejs/plugin-react';
import { defineConfig } from 'vitest/config';

/** Puertos del escritorio: tauri dev espera Vite en 1420 y el E2E sirve el build en 4174. */
const DEV_PORT = 1420;
const PREVIEW_PORT = 4174;

/** application es la capa con logica propia del frontend: 80%. El resto, 60% global. */
const CORE_THRESHOLD = { lines: 80, statements: 80, branches: 80, functions: 80 };

interface TauriConfig {
  readonly app: { readonly security: { readonly csp: Readonly<Record<string, string>> } };
}

/**
 * La CSP de produccion, leida de tauri.conf.json para no tenerla escrita dos veces.
 * El preview la manda como cabecera, asi el E2E de Playwright corre el build con
 * la misma politica que impone Tauri y falla si algo del frontend la rompe (un
 * estilo en linea, una fuente desde fuera, una peticion de red).
 */
function productionCsp(): string {
  const file = new URL('./src-tauri/tauri.conf.json', import.meta.url);
  const config = JSON.parse(readFileSync(file, 'utf8')) as TauriConfig;
  return Object.entries(config.app.security.csp)
    .map(([directive, sources]) => `${directive} ${sources}`)
    .join('; ');
}

export default defineConfig({
  plugins: [react()],
  // tauri dev ensena en la misma terminal los errores de Rust; Vite no los borra.
  clearScreen: false,
  server: { host: 'localhost', port: DEV_PORT, strictPort: true },
  preview: {
    host: '127.0.0.1',
    port: PREVIEW_PORT,
    strictPort: true,
    headers: { 'Content-Security-Policy': productionCsp() },
  },
  build: {
    outDir: 'dist',
    // Nada en linea como data: URI: la CSP solo deja fuentes de 'self'.
    assetsInlineLimit: 0,
  },
  test: {
    environment: 'jsdom',
    globals: false,
    setupFiles: ['./src/test/setup.ts'],
    include: ['src/**/*.test.{ts,tsx}'],
    clearMocks: true,
    restoreMocks: true,
    coverage: {
      provider: 'v8',
      reporter: ['text', 'html', 'lcov'],
      reportsDirectory: './coverage',
      include: ['src/**/*.{ts,tsx}'],
      exclude: ['src/app/main.tsx', 'src/test/**', 'src/**/*.test.{ts,tsx}', 'src/**/*.d.ts'],
      thresholds: {
        lines: 60,
        statements: 60,
        branches: 60,
        functions: 60,
        'src/application/**': CORE_THRESHOLD,
      },
    },
  },
});
