import { defineConfig, devices } from '@playwright/test';

/** Puerto de Playwright de la extension; escritorio usa 4174 y la web 4173. */
const PORT = 4175;
const isCi = process.env['CI'] === 'true' || process.env['CI'] === '1';
const baseURL = `http://127.0.0.1:${String(PORT)}`;

/**
 * E2E del webview del grafo contra el bundle de produccion de Vite, con un
 * `acquireVsCodeApi` falso que hace de extension host. El HTML lo genera el
 * mismo codigo que usa el host (CSP, nonce y manifiesto), asi que estos tests
 * tambien prueban que el bundle funciona con la CSP de produccion.
 *
 * Vite se lanza con node y su ruta, sin pnpm: el pnpm global de una maquina
 * puede ser otra version mayor.
 */
const VITE = 'node ./node_modules/vite/bin/vite.js';

export default defineConfig({
  testDir: './e2e',
  ...(isCi ? { workers: 1 } : {}),
  fullyParallel: true,
  forbidOnly: isCi,
  retries: isCi ? 2 : 0,
  reporter: isCi ? [['list'], ['html', { open: 'never' }]] : [['list']],
  use: { baseURL, trace: 'on-first-retry' },
  projects: [{ name: 'chromium', use: { ...devices['Desktop Chrome'] } }],
  webServer: {
    command: `${VITE} build && ${VITE} preview --host 127.0.0.1 --port ${String(PORT)} --strictPort`,
    url: `${baseURL}/manifest.json`,
    reuseExistingServer: !isCi,
    timeout: 180_000,
  },
});
