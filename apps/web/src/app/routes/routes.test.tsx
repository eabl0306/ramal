import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { createRoutesStub } from 'react-router';
import { describe, expect, it, vi } from 'vitest';

import { createHealthCheck, createHealthReport } from '../../domain/health/health-report';
import { createStaticReleaseCatalog } from '../../infrastructure/releases/static-release-catalog';
import { renderWithProviders } from '../../test/render';

import LandingRoute, { links, meta } from './landing';
import LoginRoute from './login';
import NotFoundRoute from './not-found';
import PublicLayout from './public-layout';
import SignupRoute from './signup';
import StatusRoute from './status';

const REPORT = createHealthReport({
  checks: [createHealthCheck('database', 'ok', 2)],
  version: '0.1.0',
  checkedAt: new Date('2026-09-30T10:00:00Z'),
});

function metaFor(pathname: string) {
  const args = { location: { pathname, search: '', hash: '', state: null, key: 'k' } };
  return meta(args as unknown as Parameters<typeof meta>[0]);
}

describe('modulos de ruta', () => {
  it('la landing lee el idioma de la URL de la pagina para sus metadatos', () => {
    expect(metaFor('/en/')).toContainEqual({ property: 'og:locale', content: 'en_US' });
    expect(metaFor('/')).toContainEqual({ property: 'og:locale', content: 'es_ES' });
    expect(links()).toContainEqual({ rel: 'alternate', hrefLang: 'en', href: '/en/' });
  });

  it('la landing pinta en el idioma de su URL con los datos del loader', async () => {
    const release = await createStaticReleaseCatalog().latest();
    const Stub = createRoutesStub([
      {
        path: '/',
        Component: PublicLayout,
        children: [
          { path: 'en', Component: LandingRoute, loader: () => ({ outcome: 'loaded', release }) },
        ],
      },
    ]);
    renderWithProviders(<Stub initialEntries={['/en/']} />, { language: 'en' });

    expect(await screen.findByRole('heading', { level: 1 })).toHaveTextContent('A Git client');
    expect(screen.getByTestId('download-card-linux')).toBeInTheDocument();
  });

  it('el estado pinta lo que devuelve su loader y vuelve a comprobar', async () => {
    const user = userEvent.setup();
    const loader = vi.fn(() => ({ outcome: 'reachable', report: REPORT }));
    const Stub = createRoutesStub([{ path: '/status', Component: StatusRoute, loader }]);
    renderWithProviders(<Stub initialEntries={['/status']} />);

    expect(await screen.findByTestId('health-status')).toHaveAttribute('data-status', 'ok');
    await user.click(screen.getByRole('button', { name: 'Volver a comprobar' }));
    expect(loader).toHaveBeenCalledTimes(2);
  });

  it.each([
    ['/login', 'Iniciar sesión'],
    ['/signup', 'Crear cuenta'],
    ['/nada', 'Página no encontrada'],
  ])('%s monta su pantalla', async (path, heading) => {
    const Stub = createRoutesStub([
      { path: '/login', Component: LoginRoute },
      { path: '/signup', Component: SignupRoute },
      { path: '*', Component: NotFoundRoute },
    ]);

    renderWithProviders(<Stub initialEntries={[path]} />);

    expect(await screen.findByRole('heading', { name: heading })).toBeInTheDocument();
  });
});
