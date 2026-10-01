import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it } from 'vitest';

import { renderScreen } from '../../test/render';

import { SignupScreen } from './signup-screen';

async function fillValidForm(user: ReturnType<typeof userEvent.setup>): Promise<void> {
  await user.type(screen.getByLabelText('Nombre'), 'Marta Ruiz');
  await user.type(screen.getByLabelText('Email'), 'marta@example.com');
  await user.type(screen.getByLabelText('Contraseña'), 'contrasena-larga');
  await user.click(screen.getByRole('checkbox'));
}

describe('SignupScreen', () => {
  it('ensena el primer error en la caja de error sin llamar a la API', async () => {
    const user = userEvent.setup();
    renderScreen(<SignupScreen />);

    await user.click(screen.getByRole('button', { name: 'Crear cuenta' }));

    expect(screen.getByRole('alert')).toHaveTextContent('Escribe tu nombre.');
  });

  it('pide los terminos cuando todo lo demas es valido', async () => {
    const user = userEvent.setup();
    renderScreen(<SignupScreen />);

    await user.type(screen.getByLabelText('Nombre'), 'Marta');
    await user.type(screen.getByLabelText('Email'), 'marta@example.com');
    await user.type(screen.getByLabelText('Contraseña'), 'contrasena-larga');
    await user.click(screen.getByRole('button', { name: 'Crear cuenta' }));

    expect(screen.getByRole('alert')).toHaveTextContent('Debes aceptar los términos.');
  });

  it('cambia la pista de la contrasena al llegar a 8 caracteres', async () => {
    const user = userEvent.setup();
    renderScreen(<SignupScreen />);

    expect(screen.getByText('Mínimo 8 caracteres')).toBeInTheDocument();
    await user.type(screen.getByLabelText('Contraseña'), '12345678');
    expect(screen.getByText('Contraseña válida')).toBeInTheDocument();
  });

  it('con el formulario valido contesta que todavia no esta disponible', async () => {
    const user = userEvent.setup();
    renderScreen(<SignupScreen />);

    await fillValidForm(user);
    await user.click(screen.getByRole('button', { name: 'Crear cuenta' }));

    expect(await screen.findByRole('alert')).toHaveTextContent(
      'Esta función todavía no está disponible.',
    );
  });

  it('escribir en un campo borra el error', async () => {
    const user = userEvent.setup();
    renderScreen(<SignupScreen />);

    await user.click(screen.getByRole('button', { name: 'Crear cuenta' }));
    await user.type(screen.getByLabelText('Nombre'), 'M');

    expect(screen.queryByRole('alert')).not.toBeInTheDocument();
  });

  it('se envia con Enter y en ingles ensena los textos en ingles', async () => {
    const user = userEvent.setup();
    renderScreen(<SignupScreen />, { language: 'en' });

    await user.type(screen.getByLabelText('Email'), 'x{Enter}');

    expect(screen.getByRole('alert')).toHaveTextContent('Enter your name.');
  });
});
