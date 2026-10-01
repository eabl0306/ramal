import { expect, test } from '@playwright/test';

import {
  DEGRADED_BODY,
  READY_BODY,
  USE_REAL_API,
  breakReadiness,
  mockReadiness,
} from './support/network';

test.describe('estado con la API simulada', () => {
  test.skip(USE_REAL_API, 'E2E_USE_REAL_API=1: la API real decide el estado');

  test('ensena el servicio operativo con la base de datos respondiendo', async ({ page }) => {
    await mockReadiness(page, READY_BODY, 200);
    await page.goto('/status');

    const status = page.getByTestId('health-status');
    await expect(status).toHaveAttribute('data-status', 'ok');
    await expect(status).toHaveText('Operativo');
    await expect(page.getByTestId('health-check')).toContainText('Base de datos');
    await expect(page.getByText(/Versión 0\.1\.0/u)).toBeVisible();
  });

  test('lee el 503 como degradado y marca el componente caido', async ({ page }) => {
    await mockReadiness(page, DEGRADED_BODY, 503);
    await page.goto('/status');

    await expect(page.getByTestId('health-status')).toHaveAttribute('data-status', 'degraded');
    await expect(page.getByTestId('health-check')).toHaveAttribute('data-status', 'error');
  });

  test('sin API lo dice y al volver a comprobar ensena lo nuevo', async ({ page }) => {
    await breakReadiness(page);
    await page.goto('/status');
    await expect(page.getByTestId('health-status')).toHaveAttribute('data-status', 'unreachable');

    await page.unroute('**/health/ready');
    await mockReadiness(page, READY_BODY, 200);
    await page.getByRole('button', { name: 'Volver a comprobar' }).click();

    await expect(page.getByTestId('health-status')).toHaveAttribute('data-status', 'ok');
  });
});
