import { expect, test } from '@playwright/test';

import { hostCalls, openApp } from './support/app';
import { OPENS_REPO } from './support/fixtures';

test.use({ locale: 'es-ES' });

test.describe('abrir un repositorio', () => {
  test('abre la carpeta elegida en el host y pinta su grafo', async ({ page }) => {
    const { errors } = await openApp(page, OPENS_REPO);

    await page.getByRole('button', { name: 'Abrir repositorio local' }).click();

    const grid = page.getByRole('grid', { name: 'Historial de commits' });
    await expect(grid).toBeVisible();
    await expect(grid.getByText("Merge branch 'feature/grafo'")).toBeVisible();
    await expect(grid.getByText('Primer commit')).toBeVisible();
    await expect(page.getByRole('button', { name: 'ramal' })).toHaveAttribute(
      'aria-current',
      'page',
    );
    // El grafo se dibuja en SVG a partir de los carriles que calcula el motor.
    await expect(grid.locator('svg path').first()).toBeAttached();

    // El webview nunca manda rutas: abre con el comando del host y solo pide la ventana.
    const calls = await hostCalls(page);
    expect(calls[0]).toEqual({ command: 'repo_pick_and_open' });
    expect(calls.filter((call) => call.command === 'rpc').map((call) => call.method)).toContain(
      'graph/window',
    );
    expect(calls.some((call) => call.method === 'repo/open')).toBe(false);
    expect(errors).toEqual([]);
  });

  test('si se cancela el dialogo se queda en Inicio', async ({ page }) => {
    const { errors } = await openApp(page, { ...OPENS_REPO, pick: { outcome: 'cancelled' } });

    await page.getByRole('button', { name: 'Abrir repositorio local' }).click();

    await expect(page.getByRole('heading', { name: 'Repositorios' })).toBeVisible();
    await expect(page.getByRole('alert')).toHaveCount(0);
    expect(errors).toEqual([]);
  });

  test('traduce el error del motor cuando la carpeta no es un repositorio', async ({ page }) => {
    await openApp(page, {
      ...OPENS_REPO,
      pick: {
        outcome: 'failed',
        error: { code: -32_000, message: 'not a repository', data: { kind: 'not_a_repository' } },
      },
    });

    await page.getByRole('button', { name: 'Abrir repositorio local' }).click();

    await expect(page.getByRole('alert')).toHaveText('La carpeta no es un repositorio de Git.');
  });

  test('cerrar la pestana vuelve a Inicio', async ({ page }) => {
    await openApp(page, OPENS_REPO);
    await page.getByRole('button', { name: 'Abrir repositorio local' }).click();
    await expect(page.getByRole('grid')).toBeVisible();

    await page.getByRole('button', { name: 'Cerrar' }).click();

    await expect(page.getByRole('button', { name: 'Inicio' })).toHaveAttribute(
      'aria-current',
      'page',
    );
    await expect(page.getByRole('grid')).toHaveCount(0);
  });
});
