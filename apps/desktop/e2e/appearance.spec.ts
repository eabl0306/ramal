import { expect, test } from '@playwright/test';

import { openApp } from './support/app';
import { OPENS_REPO } from './support/fixtures';

test.describe('tema e idioma', () => {
  test.use({ locale: 'es-ES' });

  test('el tema claro y el oscuro cambian los tokens del documento', async ({ page }) => {
    const { errors } = await openApp(page, OPENS_REPO);
    const html = page.locator('html');
    await expect(html).toHaveAttribute('data-theme', 'dark');
    await expect(html).toHaveAttribute('data-density', 'comfortable');
    const darkBackground = await page.evaluate(
      () => getComputedStyle(document.body).backgroundColor,
    );

    await page.getByRole('radio', { name: 'Claro' }).click();

    await expect(html).toHaveAttribute('data-theme', 'light');
    const lightBackground = await page.evaluate(
      () => getComputedStyle(document.body).backgroundColor,
    );
    expect(lightBackground).not.toBe(darkBackground);

    await page.getByRole('radio', { name: 'Oscuro' }).click();
    await expect(html).toHaveAttribute('data-theme', 'dark');
    expect(errors).toEqual([]);
  });

  test('el idioma cambia la ventana entera, tambien el grafo', async ({ page }) => {
    await openApp(page, OPENS_REPO);
    await page.getByRole('button', { name: 'Abrir repositorio local' }).click();
    await expect(page.getByRole('columnheader', { name: 'Mensaje' })).toBeVisible();

    await page.getByRole('radio', { name: 'EN' }).click();

    await expect(page.locator('html')).toHaveAttribute('lang', 'en');
    await expect(page.getByRole('button', { name: 'Home' })).toBeVisible();
    await expect(page.getByRole('columnheader', { name: 'Message' })).toBeVisible();
    await expect(page.getByRole('grid', { name: 'Commit history' })).toBeVisible();
  });

  test('tema e idioma se recuerdan al volver a abrir la app', async ({ page }) => {
    await openApp(page, OPENS_REPO);
    await page.getByRole('radio', { name: 'Claro' }).click();
    await page.getByRole('radio', { name: 'EN' }).click();

    await page.reload();

    await expect(page.locator('html')).toHaveAttribute('data-theme', 'light');
    await expect(page.getByRole('heading', { name: 'Repositories' })).toBeVisible();
  });
});

test.describe('primer arranque', () => {
  test.use({ locale: 'en-US' });

  test('sigue el idioma del sistema', async ({ page }) => {
    await openApp(page, OPENS_REPO);

    await expect(page.locator('html')).toHaveAttribute('lang', 'en');
    await expect(page.getByRole('button', { name: 'Open local repository' })).toBeVisible();
  });
});
