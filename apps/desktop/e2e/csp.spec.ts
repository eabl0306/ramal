import { expect, test } from '@playwright/test';

import { openApp } from './support/app';
import { OPENS_REPO } from './support/fixtures';

test.use({ locale: 'es-ES' });

test.describe('CSP de produccion', () => {
  test('el preview sirve la CSP de tauri.conf.json', async ({ page }) => {
    const response = await page.goto('/');

    const csp = response?.headers()['content-security-policy'] ?? '';
    expect(csp).toContain("default-src 'none'");
    expect(csp).toContain('connect-src ipc: http://ipc.localhost');
  });

  test('la app entera, con el grafo, no rompe la politica', async ({ page }) => {
    const { errors } = await openApp(page, OPENS_REPO);
    const open = page.getByRole('button', { name: 'Abrir repositorio local' });
    await expect(open).toBeVisible();

    // El estilo que inyecta React Aria entra por su hash: si la CSP lo bloqueara,
    // el navegador no le crearia hoja de estilos.
    const applied = await page.evaluate(
      () =>
        (document.getElementById('react-aria-pressable-style') as HTMLStyleElement | null)?.sheet
          ?.cssRules.length ?? 0,
    );
    expect(applied).toBeGreaterThan(0);
    await open.click();
    await expect(page.getByRole('grid')).toBeVisible();
    await page.getByRole('radio', { name: 'Claro' }).click();

    expect(errors).toEqual([]);
  });

  test('el webview no puede hablar con la red', async ({ page }) => {
    await openApp(page, OPENS_REPO);

    const blocked = await page.evaluate(async () => {
      try {
        await window.fetch('https://example.com/');
        return false;
      } catch {
        return true;
      }
    });

    expect(blocked).toBe(true);
  });
});
