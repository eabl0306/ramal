import { I18nProvider, IconProvider } from '@ramal/ui';
import { lucideIcons } from '@ramal/ui/icons/lucide';
import { render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it } from 'vitest';

import { graphWindowKey } from '../data/graph-window-query';
import { FakeEngine, fakeClient } from '../test/fake-engine';
import { graphRow, linearRows } from '../test/fixtures';
import { renderRepoView, testQueryClient } from '../test/render';

import { GraphView } from './graph-view';
import { RepoViewProvider } from './repo-view-provider';

function mount(engine: FakeEngine, pageSize?: number): ReturnType<typeof renderRepoView> {
  const client = fakeClient(engine);
  return renderRepoView(
    pageSize === undefined ? (
      <GraphView client={client} repoId="repo-1" />
    ) : (
      <GraphView client={client} repoId="repo-1" pageSize={pageSize} />
    ),
  );
}

describe('GraphView: estados', () => {
  it('ensena que carga y despues la tabla con sus cabeceras traducidas', async () => {
    const engine = new FakeEngine(linearRows(5));
    engine.holding = true;
    mount(engine);

    expect(screen.getByRole('status')).toHaveTextContent('Cargando…');
    engine.release();

    const grid = await screen.findByRole('grid', { name: 'Historial de commits' });
    const headers = within(grid).getAllByRole('columnheader');
    expect(headers.map((header) => header.textContent)).toEqual([
      'Rama / tag',
      'Grafo',
      'Mensaje',
      'Autor',
      'Fecha',
      'SHA',
    ]);
    expect(grid).toHaveAttribute('aria-colcount', '6');
    expect(grid).toHaveAttribute('aria-rowcount', '6');
  });

  it('pide la primera ventana con el repositorio y el tamano de pagina por defecto', async () => {
    const engine = new FakeEngine(linearRows(3));
    mount(engine);

    await screen.findByRole('grid');
    expect(engine.calls[0]).toEqual({ repoId: 'repo-1', offset: 0, limit: 200 });
  });

  it('dice que el repositorio no tiene commits', async () => {
    mount(new FakeEngine([]));

    const message = await screen.findByText('Este repositorio todavía no tiene commits.');
    expect(message.closest('[role="status"]')).not.toBeNull();
    expect(screen.queryByRole('grid')).not.toBeInTheDocument();
  });

  it('traduce el error del motor y vuelve a pedir al reintentar', async () => {
    const engine = new FakeEngine(linearRows(3));
    engine.failAt(0, 'repo_not_open');
    mount(engine);

    const alert = await screen.findByRole('alert');
    expect(alert).toHaveTextContent('No se ha podido cargar el historial.');
    expect(alert).toHaveTextContent('El repositorio no está abierto.');

    await userEvent.click(within(alert).getByRole('button', { name: 'Reintentar' }));

    expect(await screen.findByRole('grid')).toBeInTheDocument();
    expect(engine.calls).toHaveLength(2);
  });

  it('crea su propio QueryClient si el host no le pasa uno', async () => {
    render(
      <I18nProvider locale="es">
        <IconProvider icons={lucideIcons}>
          <RepoViewProvider>
            <GraphView client={fakeClient(new FakeEngine(linearRows(2)))} repoId="r" />
          </RepoViewProvider>
        </IconProvider>
      </I18nProvider>,
    );

    expect(await screen.findByRole('grid')).toBeInTheDocument();
  });

  it('ensena tal cual una fecha que no se puede leer', async () => {
    mount(new FakeEngine([graphRow('a', { authoredAt: 'sin fecha' })]));

    const time = await screen.findByText('sin fecha');
    expect(time).toHaveAttribute('title', 'sin fecha');
  });

  it('usa el QueryClient del host para guardar la ventana', async () => {
    const queryClient = testQueryClient();
    renderRepoView(<GraphView client={fakeClient(new FakeEngine(linearRows(2)))} repoId="r" />, {
      queryClient,
    });

    await screen.findByRole('grid');
    expect(queryClient.getQueryData(graphWindowKey('r', 200))).toBeDefined();
  });
});

describe('GraphView: paginas al bajar', () => {
  it('pide paginas mientras las filas visibles se acercan al final', async () => {
    const engine = new FakeEngine(linearRows(50));
    mount(engine, 20);

    const grid = await screen.findByRole('grid');
    await waitFor(() => {
      expect(grid).toHaveAttribute('aria-rowcount', '51');
    });
    expect(engine.calls.map((call) => call.offset)).toEqual([0, 20, 40]);
  });

  it('mientras quedan paginas el total de filas es desconocido', async () => {
    const engine = new FakeEngine(linearRows(300));
    mount(engine, 100);

    const grid = await screen.findByRole('grid');
    expect(grid).toHaveAttribute('aria-rowcount', '-1');
  });

  it('ensena el fallo de una pagina al final de la tabla y la vuelve a pedir', async () => {
    const engine = new FakeEngine(linearRows(30));
    engine.failAt(10, 'git_failed');
    mount(engine, 10);

    const alert = await screen.findByRole('alert');
    expect(alert).toHaveTextContent('Git ha devuelto un error.');
    expect(screen.getAllByRole('row').length).toBeGreaterThan(10);

    await userEvent.click(screen.getByRole('button', { name: 'Reintentar' }));

    await waitFor(() => {
      expect(screen.getByRole('grid')).toHaveAttribute('aria-rowcount', '31');
    });
  });
});
