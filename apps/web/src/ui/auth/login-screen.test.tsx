import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it } from 'vitest';

import type { AuthGateway } from '../../application/ports/auth-gateway';
import { renderScreen, testPorts } from '../../test/render';

import { LoginScreen } from './login-screen';

describe('LoginScreen', () => {
  it('valida el email antes de nada', async () => {
    const user = userEvent.setup();
    renderScreen(<LoginScreen />);

    await user.type(screen.getByLabelText('Email'), 'no-es-email');
    await user.click(screen.getByRole('button', { name: 'Iniciar sesión' }));

    expect(screen.getByRole('alert')).toHaveTextContent('Escribe un email válido.');
  });

  it('pide la contrasena pero no su longitud minima', async () => {
    const user = userEvent.setup();
    renderScreen(<LoginScreen />);

    await user.type(screen.getByLabelText('Email'), 'marta@ramal.dev');
    await user.click(screen.getByRole('button', { name: 'Iniciar sesión' }));
    expect(screen.getByRole('alert')).toHaveTextContent('Escribe tu contraseña.');

    await user.type(screen.getByLabelText('Contraseña'), 'corta');
    await user.click(screen.getByRole('button', { name: 'Iniciar sesión' }));
    expect(await screen.findByRole('alert')).toHaveTextContent(
      'Esta función todavía no está disponible.',
    );
  });

  it('los proveedores contestan lo mismo que el formulario', async () => {
    const user = userEvent.setup();
    renderScreen(<LoginScreen />);

    await user.click(screen.getByRole('button', { name: 'Continuar con GitHub' }));

    expect(await screen.findByRole('alert')).toHaveTextContent(
      'Esta función todavía no está disponible.',
    );
  });

  it('si la API acepta, avisa de la sesion y quita el error', async () => {
    const user = userEvent.setup();
    const auth: AuthGateway = {
      signIn: (credentials) => Promise.resolve({ ok: true, session: { email: credentials.email } }),
      signUp: () => Promise.resolve({ ok: false, failure: 'unknown' }),
      signInWithProvider: () => Promise.reject(new Error('sin red')),
    };
    renderScreen(<LoginScreen />, { ports: testPorts({ auth }) });

    await user.click(screen.getByRole('button', { name: 'Continuar con GitLab' }));
    expect(await screen.findByRole('alert')).toHaveTextContent('No hay conexión con Ramal.');

    await user.type(screen.getByLabelText('Email'), 'marta@ramal.dev');
    await user.type(screen.getByLabelText('Contraseña'), 'x');
    await user.click(screen.getByRole('button', { name: 'Iniciar sesión' }));

    expect(await screen.findByText('Sesión iniciada como marta@ramal.dev')).toBeInTheDocument();
    expect(screen.queryByRole('alert')).not.toBeInTheDocument();
  });
});
