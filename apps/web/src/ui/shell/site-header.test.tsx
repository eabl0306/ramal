import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import type { ReactElement } from 'react';
import { useLocation } from 'react-router';
import { describe, expect, it } from 'vitest';

import { renderScreen, testPorts } from '../../test/render';

import { SiteFooter, SiteHeader } from './site-header';

/** Ensena la URL actual para comprobar a donde lleva el conmutador de idioma. */
function CurrentPath(): ReactElement {
  const location = useLocation();
  return <output data-testid="path">{`${location.pathname}${location.hash}`}</output>;
}

describe('SiteHeader', () => {
  it('en la landing las secciones son anclas de la propia pagina', () => {
    renderScreen(<SiteHeader />, { path: '/' });

    expect(screen.getByRole('link', { name: 'Descargar' })).toHaveAttribute('href', '#download');
    expect(screen.getByRole('link', { name: 'Crear cuenta' })).toHaveAttribute('href', '/signup');
  });

  it('fuera de la landing las secciones llevan a la landing del idioma actual', () => {
    renderScreen(<SiteHeader />, { path: '/login', language: 'en' });

    expect(screen.getByRole('link', { name: 'Download' })).toHaveAttribute('href', '/en/#download');
    expect(screen.getByRole('link', { name: 'Log in' })).toHaveAttribute('href', '/login');
  });

  it('cambiar de idioma en la landing lleva a la otra landing y lo recuerda', async () => {
    const user = userEvent.setup();
    const ports = testPorts();
    renderScreen(
      <>
        <SiteHeader />
        <CurrentPath />
      </>,
      { path: '/#download', ports },
    );

    await user.click(screen.getByRole('radio', { name: 'EN' }));

    expect(screen.getByTestId('path')).toHaveTextContent('/en/#download');
    expect(ports.preferences.read().language).toBe('en');
  });

  it('cambiar de idioma fuera de la landing solo guarda la preferencia', async () => {
    const user = userEvent.setup();
    const ports = testPorts();
    renderScreen(
      <>
        <SiteHeader />
        <CurrentPath />
      </>,
      { path: '/status', ports },
    );

    await user.click(screen.getByRole('radio', { name: 'EN' }));

    expect(screen.getByTestId('path')).toHaveTextContent('/status');
    expect(ports.preferences.read().language).toBe('en');
  });

  it('el conmutador de tema guarda lo elegido', async () => {
    const user = userEvent.setup();
    const ports = testPorts();
    renderScreen(<SiteHeader />, { ports });

    await user.click(screen.getByRole('radio', { name: 'Claro' }));

    expect(ports.preferences.read().theme).toBe('light');
    expect(screen.getByRole('radio', { name: 'Claro' })).toBeChecked();
  });
});

describe('SiteFooter', () => {
  it('enlaza la pagina de estado', () => {
    renderScreen(<SiteFooter />);

    expect(screen.getByRole('link', { name: 'Estado del servicio' })).toHaveAttribute(
      'href',
      '/status',
    );
  });
});
