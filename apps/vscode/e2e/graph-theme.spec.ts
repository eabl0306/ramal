import { type Locator, type Page, expect, test } from '@playwright/test';

import { TWO_COMMITS, openGraphWebview } from './fake-host';

/**
 * El grafo con los colores del editor. VS Code pone las variables --vscode-* en
 * <html> y la clase del tema en <body>; aqui se ponen a mano con colores de
 * carril, que son tokens, para poder comparar sin escribir un color.
 */

async function setEditorTheme(page: Page, vars: Readonly<Record<string, string>>): Promise<void> {
  await page.evaluate((entries) => {
    for (const [name, value] of Object.entries(entries)) {
      document.documentElement.style.setProperty(name, value);
    }
  }, vars);
}

/** El color calculado de un token, leido de un elemento de prueba. */
async function resolved(page: Page, token: string): Promise<string> {
  return page.evaluate((value) => {
    const probe = document.createElement('span');
    probe.style.color = value;
    document.body.append(probe);
    const color = getComputedStyle(probe).color;
    probe.remove();
    return color;
  }, token);
}

function rowWith(page: Page, text: string): Locator {
  return page.getByRole('row').filter({ hasText: text });
}

async function openGraph(page: Page): Promise<Locator> {
  await openGraphWebview(page, { locale: 'es', window: TWO_COMMITS, engineDown: false });
  await expect(page.getByText('Primer commit')).toBeVisible();
  await setEditorTheme(page, {
    '--vscode-list-activeSelectionBackground': 'var(--r-lane-0)',
    '--vscode-list-activeSelectionForeground': 'var(--r-lane-3)',
    '--vscode-list-inactiveSelectionBackground': 'var(--r-lane-1)',
    '--vscode-contrastActiveBorder': 'var(--r-lane-2)',
  });
  return rowWith(page, 'Primer commit');
}

test('la fila elegida usa la seleccion activa con foco y la inactiva sin el', async ({ page }) => {
  const row = await openGraph(page);
  await row.getByText('Primer commit').click();

  await expect(row).toHaveCSS('background-color', await resolved(page, 'var(--r-lane-0)'));
  await expect(row).toHaveCSS('color', await resolved(page, 'var(--r-lane-3)'));

  await page.evaluate(() => {
    (document.activeElement as HTMLElement | null)?.blur();
  });
  await expect(row).toHaveCSS('background-color', await resolved(page, 'var(--r-lane-1)'));
});

test('en alto contraste la elegida y la de debajo del raton se marcan con borde', async ({
  page,
}) => {
  const row = await openGraph(page);
  await page.evaluate(() => {
    document.body.classList.add('vscode-high-contrast');
  });
  await row.getByText('Primer commit').click();
  const border = await resolved(page, 'var(--r-lane-2)');

  await expect(row).toHaveCSS('outline-style', 'solid');
  await expect(row).toHaveCSS('outline-color', border);
  const other = rowWith(page, 'Pintar el grafo en VS Code');
  await other.getByText('Pintar el grafo en VS Code').hover();
  await expect(other).toHaveCSS('outline-style', 'dashed');
  await expect(other).toHaveCSS('outline-color', border);
});

test('la fecha lleva dia, mes y hora y cabe en la columna de VS Code', async ({ page }) => {
  const row = await openGraph(page);
  const time = row.locator('time');

  await expect(time).toHaveText(/^\d{1,2}\/\d{1,2} \d{2}:\d{2}$/u);
  const fits = await time.evaluate((element) => {
    const cell = element.parentElement;
    return cell !== null && cell.scrollWidth <= cell.clientWidth;
  });
  expect(fits).toBe(true);
});
