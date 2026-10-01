import path from 'node:path';
import { fileURLToPath } from 'node:url';

import type { Page } from '@playwright/test';

import type { E2eCall, E2eScenario } from './scenario';

/** Donde deja global-setup el host simulado empaquetado (ignorado por git). */
export const IPC_MOCK_DIR = fileURLToPath(new URL('../../node_modules/.tmp/e2e', import.meta.url));
export const IPC_MOCK_FILE = 'ipc-mock.js';

export interface OpenedApp {
  /** Errores de la consola y de la pagina, incluidas las violaciones de la CSP. */
  readonly errors: string[];
}

/**
 * Carga el build de produccion con el host simulado: primero el escenario, despues
 * `mockIPC`, despues la app. El preview sirve la CSP de tauri.conf.json, asi que
 * cualquier cosa que la rompa aparece en `errors`.
 */
export async function openApp(page: Page, scenario: E2eScenario): Promise<OpenedApp> {
  const errors: string[] = [];
  page.on('console', (message) => {
    if (message.type() === 'error') errors.push(message.text());
  });
  page.on('pageerror', (error) => {
    errors.push(error.message);
  });
  await page.addInitScript((value) => {
    window.__RAMAL_E2E__ = value;
  }, scenario);
  await page.addInitScript({ path: path.join(IPC_MOCK_DIR, IPC_MOCK_FILE) });
  await page.goto('/');
  return { errors };
}

/** Lo que el webview ha pedido al host hasta ahora. */
export function hostCalls(page: Page): Promise<E2eCall[]> {
  return page.evaluate(() => window.__RAMAL_E2E_CALLS__ ?? []);
}
