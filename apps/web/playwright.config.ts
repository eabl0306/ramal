import { defineConfig, devices } from '@playwright/test';

/**
 * Dos modos. Por defecto, contra el build de produccion servido como lo sirve
 * nginx (scripts/preview-server.mjs) y con la red de la API simulada en cada
 * test. Con E2E_USE_REAL_API=1 no se levanta nada: E2E_BASE_URL apunta al
 * gateway de la pila Docker y se entra por la puerta de verdad (just e2e-stack).
 */
const PREVIEW_PORT = 4173;
const isCi = process.env['CI'] === 'true' || process.env['CI'] === '1';
const useRealApi = process.env['E2E_USE_REAL_API'] === '1';
const baseURL = process.env['E2E_BASE_URL'] ?? `http://127.0.0.1:${String(PREVIEW_PORT)}`;

export default defineConfig({
  testDir: './e2e',
  ...(isCi ? { workers: 1 } : {}),
  fullyParallel: true,
  forbidOnly: isCi,
  retries: isCi ? 2 : 0,
  reporter: isCi ? [['list'], ['html', { open: 'never' }]] : [['list']],
  // Navegador en castellano: sin preferencia guardada, las rutas SPA siguen el
  // idioma del navegador, y el castellano es el idioma principal del producto.
  use: { baseURL, locale: 'es-ES', trace: 'on-first-retry' },
  projects: [{ name: 'chromium', use: { ...devices['Desktop Chrome'] } }],
  ...(useRealApi
    ? {}
    : {
        webServer: {
          command: 'react-router build && node scripts/preview-server.mjs',
          url: baseURL,
          reuseExistingServer: !isCi,
          timeout: 180_000,
        },
      }),
});
