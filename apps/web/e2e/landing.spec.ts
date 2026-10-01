import { expect, test } from '@playwright/test';

import { USER_AGENTS } from './support/visitors';

const DESKTOPS = [
  { os: 'mac', name: 'macOS', userAgent: USER_AGENTS.mac },
  { os: 'windows', name: 'Windows', userAgent: USER_AGENTS.windows },
  { os: 'linux', name: 'Linux', userAgent: USER_AGENTS.linux },
] as const;

for (const desktop of DESKTOPS) {
  test.describe(`landing en ${desktop.name}`, () => {
    test.use({ userAgent: desktop.userAgent });

    test('ofrece su instalador y destaca su tarjeta como "Tu sistema"', async ({ page }) => {
      await page.goto('/');

      await expect(
        page.getByRole('button', { name: `Descargar para ${desktop.name}` }),
      ).toBeVisible();
      const card = page.getByTestId(`download-card-${desktop.os}`);
      await expect(card).toHaveAttribute('data-highlighted', 'true');
      await expect(card.getByText('Tu sistema')).toBeVisible();
      await expect(page.getByText('Tu sistema')).toHaveCount(1);
    });
  });
}

test.describe('landing en un movil', () => {
  test.use({ userAgent: USER_AGENTS.iphone });

  test('no destaca ninguna tarjeta y avisa de que Ramal es de escritorio', async ({ page }) => {
    await page.goto('/');

    await expect(page.getByRole('note')).toContainText('Ramal es una app de escritorio');
    await expect(page.getByText('Tu sistema')).toHaveCount(0);
    await expect(page.getByRole('button', { name: /Descargar para/u })).toHaveCount(0);
  });
});

test('la landing llega prerenderizada: el HTML ya trae el contenido', async ({ request }) => {
  const spanish = await request.get('/');
  const english = await request.get('/en/');

  expect(spanish.headers()['content-type']).toContain('text/html');
  expect(await spanish.text()).toContain('Un cliente de Git para todos tus repositorios');
  expect(await english.text()).toContain('A Git client for all your repositories');
  expect(await english.text()).toContain('<html lang="en"');
});
