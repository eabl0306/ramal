import { expect, test } from '@playwright/test';

import { USE_REAL_API } from './support/network';

/**
 * Contra la pila Docker, entrando por el gateway (just e2e-stack). Es lo que un
 * test de una sola pieza no ve: que la web llega a la API por el mismo origen y
 * que una ruta de la API no acaba en la SPA (la leccion de Gastonomo, ADR 0007).
 */
test.describe('pila real a traves del gateway', () => {
  test.skip(!USE_REAL_API, 'necesita la pila levantada: just e2e-stack');

  test('la pagina de estado ensena lo que responde /health/ready', async ({ page }) => {
    await page.goto('/status');

    const status = page.getByTestId('health-status');
    await expect(status).toHaveAttribute('data-status', 'ok');
    await expect(page.getByTestId('health-check')).toHaveAttribute('data-status', 'ok');
  });

  test('una ruta desconocida de /v1 la contesta la API y no la SPA', async ({ request }) => {
    const response = await request.get('/v1/ruta-que-no-existe');

    expect(response.status()).toBe(404);
    expect(response.headers()['content-type']).toContain('application/problem+json');
  });

  test('la landing en ingles llega prerenderizada por el gateway', async ({ page }) => {
    await page.goto('/en/');

    await expect(page.getByRole('heading', { level: 1 })).toContainText('A Git client');
  });
});
