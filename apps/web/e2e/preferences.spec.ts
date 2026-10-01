import { type Page, expect, test } from '@playwright/test';

/**
 * La landing llega prerenderizada y no responde a los clics hasta hidratar. El
 * boton "Descargar para..." solo existe despues, porque el sistema se detecta en
 * el navegador: es la senal de que ya se puede tocar la pagina.
 */
async function openLanding(page: Page, path: string): Promise<void> {
  await page.goto(path);
  await expect(page.getByRole('button', { name: /Descargar para|Download for/u })).toBeVisible();
}

test('cambiar a ingles lleva a /en/ y el idioma sigue en las paginas SPA', async ({ page }) => {
  await openLanding(page, '/');
  await expect(page.getByRole('heading', { level: 1 })).toContainText('Un cliente de Git');

  await page.getByRole('radio', { name: 'EN' }).click();

  await expect(page).toHaveURL(/\/en\/$/u);
  await expect(page.getByRole('heading', { level: 1 })).toContainText('A Git client');
  await expect(page.locator('html')).toHaveAttribute('lang', 'en');

  await page.getByRole('link', { name: 'Log in' }).click();
  await expect(page).toHaveURL(/\/login$/u);
  await expect(page.getByRole('heading', { name: 'Log in' })).toBeVisible();

  await page.reload();
  await expect(page.getByRole('heading', { name: 'Log in' })).toBeVisible();
});

test('en una pagina SPA el conmutador cambia el idioma sin cambiar de URL', async ({ page }) => {
  await page.goto('/signup');
  await expect(page.getByRole('heading', { name: 'Crear cuenta' })).toBeVisible();

  await page.getByRole('radio', { name: 'EN' }).click();

  await expect(page).toHaveURL(/\/signup$/u);
  await expect(page.getByRole('heading', { name: 'Create account' })).toBeVisible();
  await expect(page.locator('html')).toHaveAttribute('lang', 'en');
});

test.describe('tema', () => {
  test.use({ colorScheme: 'dark' });

  test('el claro se aplica y se recuerda al recargar', async ({ page }) => {
    await openLanding(page, '/');
    await expect(page.locator('html')).toHaveAttribute('data-theme', 'dark');

    await page.getByRole('radio', { name: 'Claro' }).click();
    await expect(page.locator('html')).toHaveAttribute('data-theme', 'light');

    await page.reload();
    await expect(page.locator('html')).toHaveAttribute('data-theme', 'light');
  });
});

test.describe('tema del sistema', () => {
  test.use({ colorScheme: 'light' });

  test('sin nada elegido sigue al sistema', async ({ page }) => {
    await openLanding(page, '/');

    await expect(page.locator('html')).toHaveAttribute('data-theme', 'light');
  });
});
