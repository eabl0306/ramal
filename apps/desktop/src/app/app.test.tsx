import { render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it } from 'vitest';

import { APPEARANCE_KEY } from '../infrastructure/storage/local-appearance-store';
import { FakeTauriHost, graphRow } from '../test/fake-host';

import { App } from './app';
import { createContainer } from './composition-root';

const HISTORY = [
  graphRow(0, 'Tercer commit'),
  graphRow(1, 'Segundo commit'),
  graphRow(2, 'Primer commit'),
];

function mount(host = new FakeTauriHost(HISTORY), systemLanguage = 'es-ES') {
  localStorage.clear();
  const root = document.createElement('html');
  const container = createContainer({ invoke: host.invoke, storage: localStorage, systemLanguage });
  render(<App container={container} root={root} />);
  return { host, root };
}

async function openRepository(): Promise<void> {
  await userEvent.click(screen.getByRole('button', { name: 'Abrir repositorio local' }));
}

describe('App: abrir un repositorio', () => {
  it('arranca en Inicio con la entrada para abrir un repositorio', () => {
    mount();

    expect(screen.getByRole('heading', { name: 'Repositorios' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Inicio' })).toHaveAttribute('aria-current', 'page');
  });

  it('abre la pestana del repositorio y pinta su grafo con el motor', async () => {
    const { host } = mount();

    await openRepository();

    const tab = screen.getByRole('button', { name: 'ramal' });
    expect(tab).toHaveAttribute('aria-current', 'page');
    expect(tab).toHaveAttribute('title', '/home/ana/code/ramal');
    const grid = await screen.findByRole('grid', { name: 'Historial de commits' });
    expect(within(grid).getByText('Segundo commit')).toBeInTheDocument();
    expect(host.requests[0]).toMatchObject({
      method: 'graph/window',
      params: { repoId: 'repo-1', offset: 0 },
    });
  });

  it('se queda en Inicio sin error si el usuario cancela el dialogo', async () => {
    const host = new FakeTauriHost(HISTORY);
    host.pick = null;
    mount(host);

    await openRepository();

    expect(screen.getByRole('heading', { name: 'Repositorios' })).toBeInTheDocument();
    expect(screen.queryByRole('alert')).not.toBeInTheDocument();
    expect(host.requests).toEqual([]);
  });

  it('traduce el error del motor si la carpeta no es un repositorio', async () => {
    const host = new FakeTauriHost(HISTORY);
    host.pick = {
      reject: { code: -32_000, message: 'not a repository', data: { kind: 'not_a_repository' } },
    };
    mount(host);

    await openRepository();

    expect(await screen.findByRole('alert')).toHaveTextContent(
      'La carpeta no es un repositorio de Git.',
    );
  });

  it('vuelve al grafo desde Inicio con la pestana del repositorio', async () => {
    mount();
    await openRepository();
    await screen.findByRole('grid');

    await userEvent.click(screen.getByRole('button', { name: 'Inicio' }));
    expect(screen.queryByRole('grid')).not.toBeInTheDocument();

    await userEvent.click(screen.getByRole('button', { name: 'ramal' }));
    expect(await screen.findByRole('grid')).toBeInTheDocument();
  });

  it('cerrar la pestana vuelve a Inicio y volver a abrir no la duplica', async () => {
    mount();
    await openRepository();
    await screen.findByRole('grid');

    await userEvent.click(screen.getByRole('button', { name: 'Inicio' }));
    await openRepository();
    expect(screen.getAllByRole('button', { name: 'ramal' })).toHaveLength(1);

    await userEvent.click(screen.getByRole('button', { name: 'Cerrar' }));
    expect(screen.queryByRole('button', { name: 'ramal' })).not.toBeInTheDocument();
    expect(screen.getByRole('heading', { name: 'Repositorios' })).toBeInTheDocument();
  });
});

describe('App: apariencia', () => {
  it('cambia a tema claro, lo pone en el documento y lo guarda', async () => {
    const { root } = mount();
    expect(root.dataset['theme']).toBe('dark');

    await userEvent.click(screen.getByRole('radio', { name: 'Claro' }));

    expect(root.dataset['theme']).toBe('light');
    expect(JSON.parse(localStorage.getItem(APPEARANCE_KEY) ?? '{}')).toEqual({
      theme: 'light',
      locale: 'es',
    });
  });

  it('cambia el idioma de toda la ventana', async () => {
    const { root } = mount();

    await userEvent.click(screen.getByRole('radio', { name: 'EN' }));

    expect(root.lang).toBe('en');
    expect(screen.getByRole('button', { name: 'Home' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Open local repository' })).toBeInTheDocument();
  });

  it('el primer arranque sigue el idioma del sistema', () => {
    const { root } = mount(new FakeTauriHost(HISTORY), 'en-US');

    expect(root.lang).toBe('en');
    expect(screen.getByRole('heading', { name: 'Repositories' })).toBeInTheDocument();
  });
});
