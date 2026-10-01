import { expect, test } from '@playwright/test';

test('el registro valida campo a campo y despues dice que aun no esta disponible', async ({
  page,
}) => {
  await page.goto('/signup');
  const submit = page.getByRole('button', { name: 'Crear cuenta' });
  const error = page.getByTestId('auth-error');

  await submit.click();
  await expect(error).toHaveText('Escribe tu nombre.');

  await page.getByLabel('Nombre').fill('Marta Ruiz');
  await expect(error).toHaveCount(0);
  await submit.click();
  await expect(error).toHaveText('Escribe un email válido.');

  await page.getByLabel('Email').fill('marta@example.com');
  await page.getByLabel('Contraseña').fill('corta');
  await expect(page.getByText('Mínimo 8 caracteres')).toBeVisible();
  await submit.click();
  await expect(error).toHaveText('La contraseña debe tener al menos 8 caracteres.');

  await page.getByLabel('Contraseña').fill('contrasena-larga');
  await expect(page.getByText('Contraseña válida')).toBeVisible();
  await submit.click();
  await expect(error).toHaveText('Debes aceptar los términos.');

  await page.getByRole('checkbox').check();
  await submit.click();
  await expect(error).toHaveText('Esta función todavía no está disponible.');
});

test('entrar no exige longitud minima y contesta lo mismo', async ({ page }) => {
  await page.goto('/login');

  await page.getByLabel('Email').fill('marta@example.com');
  await page.getByLabel('Contraseña').fill('corta');
  await page.getByLabel('Contraseña').press('Enter');

  await expect(page.getByTestId('auth-error')).toHaveText(
    'Esta función todavía no está disponible.',
  );
});
