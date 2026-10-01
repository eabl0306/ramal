import { type Locator, expect, test } from '@playwright/test';

/**
 * Medidas del prototipo web (docs/design/handoff/Ramal Web.dc.html) tomadas en
 * un navegador de verdad: jsdom no aplica los CSS Modules, asi que un test
 * unitario solo puede leer la hoja. Aqui se mide la caja que se pinta.
 */

interface Box {
  readonly width: number;
  readonly height: number;
  readonly radius: string;
  readonly gap: string;
}

async function box(locator: Locator): Promise<Box> {
  return locator.evaluate((element) => {
    const rect = element.getBoundingClientRect();
    const style = getComputedStyle(element);
    return {
      width: Math.round(rect.width),
      height: Math.round(rect.height),
      radius: style.borderTopLeftRadius,
      gap: style.columnGap,
    };
  });
}

test('los conmutadores de la cabecera miden lo del prototipo', async ({ page }) => {
  await page.goto('/login');
  const theme = page.getByRole('radiogroup', { name: 'Tema' });
  const language = page.getByRole('radiogroup', { name: 'Idioma' });

  // Dos segmentos de 30 pegados, 2 px de relleno y 1 px de borde por lado.
  expect(await box(theme)).toMatchObject({ width: 66, height: 32, radius: '8px', gap: '0px' });
  expect(await box(language)).toMatchObject({ height: 32, radius: '8px', gap: '0px' });
  expect(await box(theme.getByRole('radio', { name: 'Claro' }))).toMatchObject({
    width: 30,
    height: 26,
    radius: '6px',
  });
  // Sin fondo: el mismo color calculado que un elemento sin estilos a su lado.
  const transparent = await theme.evaluate((element) => {
    const probe = document.createElement('span');
    element.after(probe);
    const same =
      getComputedStyle(element).backgroundColor === getComputedStyle(probe).backgroundColor;
    probe.remove();
    return same;
  });
  expect(transparent).toBe(true);
});

test('la tarjeta de entrar tiene los controles del prototipo', async ({ page }) => {
  await page.goto('/login');

  const provider = page.getByRole('button', { name: 'Continuar con GitHub' });
  expect(await box(provider)).toMatchObject({ height: 42, radius: '10px', gap: '10px' });
  expect(await box(provider.locator('[role="img"]'))).toMatchObject({
    width: 20,
    height: 20,
    radius: '6px',
  });
  const google = page.getByRole('button', { name: 'Continuar con Google' });
  expect(await box(google.locator('span').first())).toMatchObject({ width: 20, height: 20 });

  expect(await box(page.getByLabel('Email'))).toMatchObject({ height: 42, radius: '10px' });
  const submit = page.getByRole('button', { name: 'Iniciar sesión' });
  expect(await box(submit)).toMatchObject({ height: 44, radius: '10px' });
  await expect(submit).toHaveCSS('font-size', '15px');
});

test('el boton principal de cada descarga lleva las esquinas de 10 del prototipo', async ({
  page,
}) => {
  await page.goto('/');
  const card = page.getByTestId('download-card-mac');

  expect(await box(card.getByRole('button', { name: 'Apple Silicon (.dmg)' }))).toMatchObject({
    height: 42,
    radius: '10px',
  });
});
