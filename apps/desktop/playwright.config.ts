import { defineConfig, devices } from '@playwright/test';

/**
 * Flujos de interfaz contra el build de produccion de Vite, con el IPC de Tauri
 * simulado por `mockIPC` (e2e/support/ipc-mock.ts) y la CSP de tauri.conf.json en
 * el servidor de preview. El binario real con el motor lo prueba el E2E nativo
 * (wdio/).
 */
const PREVIEW_PORT = 4174;
const isCi = process.env['CI'] === 'true' || process.env['CI'] === '1';
const baseURL = `http://127.0.0.1:${String(PREVIEW_PORT)}`;
/** El tamano de ventana de tauri.conf.json, el del diseno. */
const WINDOW = { width: 1560, height: 960 };

export default defineConfig({
  testDir: './e2e',
  globalSetup: './e2e/global-setup.ts',
  ...(isCi ? { workers: 1 } : {}),
  fullyParallel: true,
  forbidOnly: isCi,
  retries: isCi ? 2 : 0,
  reporter: isCi ? [['list'], ['html', { open: 'never' }]] : [['list']],
  use: { baseURL, trace: 'on-first-retry' },
  projects: [{ name: 'chromium', use: { ...devices['Desktop Chrome'], viewport: WINDOW } }],
  webServer: {
    // Corre desde el script e2e del paquete, con node_modules/.bin en el PATH.
    command: 'vite build && vite preview',
    url: baseURL,
    reuseExistingServer: !isCi,
    timeout: 180_000,
  },
});
