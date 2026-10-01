import { expect, test } from '@playwright/test';

import {
  TWO_COMMITS,
  bringEngineUp,
  cspViolations,
  openGraphWebview,
  postedMessages,
} from './fake-host';

test.describe('webview del grafo', () => {
  test('pinta los commits del motor en castellano y con la densidad de VS Code', async ({
    page,
  }) => {
    await openGraphWebview(page, { locale: 'es', window: TWO_COMMITS, engineDown: false });
    const grid = page.getByRole('grid', { name: 'Historial de commits' });
    await expect(grid).toBeVisible();
    await expect(page.getByText('Pintar el grafo en VS Code')).toBeVisible();
    await expect(page.getByText('Primer commit')).toBeVisible();
    await expect(page.getByText('main', { exact: true })).toBeVisible();
    const rowHeight = await page
      .locator('[data-density="compact"]')
      .first()
      .evaluate((element) => getComputedStyle(element).getPropertyValue('--r-row-h').trim());
    expect(rowHeight).toBe('28px');
  });

  test('sigue el idioma de VS Code: en ingles, los textos en ingles', async ({ page }) => {
    await openGraphWebview(page, { locale: 'en', window: TWO_COMMITS, engineDown: false });
    await expect(page.getByRole('grid', { name: 'Commit history' })).toBeVisible();
    expect(await page.locator('html').getAttribute('lang')).toBe('en');
  });

  test('funciona con la CSP de produccion sin una sola violacion', async ({ page }) => {
    const errors: string[] = [];
    page.on('console', (message) => {
      if (message.type() === 'error') errors.push(message.text());
    });
    await openGraphWebview(page, { locale: 'es', window: TWO_COMMITS, engineDown: false });
    await expect(page.getByText('Primer commit')).toBeVisible();
    expect(await cspViolations(page)).toEqual([]);
    expect(errors).toEqual([]);
  });

  test('solo pide graph/window de su repo y nunca manda rutas', async ({ page }) => {
    await openGraphWebview(page, { locale: 'es', window: TWO_COMMITS, engineDown: false });
    await expect(page.getByText('Primer commit')).toBeVisible();
    const posted = await postedMessages(page);
    expect(posted[0]).toEqual({ type: 'webview/ready', surface: 'graph' });
    const requests = posted.slice(1) as {
      type: string;
      request: { method: string; params: Record<string, unknown> };
    }[];
    expect(requests.length).toBeGreaterThan(0);
    for (const message of requests) {
      expect(message.type).toBe('rpc/request');
      expect(message.request.method).toBe('graph/window');
      expect(Object.keys(message.request.params).sort()).toEqual(['limit', 'offset', 'repoId']);
      expect(message.request.params['repoId']).toBe('repo-e2e');
    }
  });

  test('si el motor no esta, lo dice y reintentar vuelve a pedir el grafo', async ({ page }) => {
    await openGraphWebview(page, { locale: 'es', window: TWO_COMMITS, engineDown: true });
    const alert = page.getByRole('alert');
    await expect(alert).toContainText('El motor de Ramal no está disponible.', { timeout: 15_000 });
    await bringEngineUp(page);
    await alert.getByRole('button', { name: 'Reintentar' }).click();
    await expect(page.getByText('Primer commit')).toBeVisible();
  });
});
