import type { GraphRow } from '@ramal/bridge';
import { codicons } from '@ramal/ui/icons/codicons';
import { screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import { FakeEngine, fakeClient } from '../test/fake-engine';
import { graphEdge, graphRow, linearRows, refLabel } from '../test/fixtures';
import { type RenderOptions, renderRepoView } from '../test/render';

import { GraphView, type GraphViewProps } from './graph-view';

type ViewOptions = Omit<GraphViewProps, 'client' | 'repoId'>;

async function mountGrid(
  history: readonly GraphRow[],
  props: ViewOptions = {},
  options: RenderOptions = {},
): Promise<{ grid: HTMLElement; container: HTMLElement }> {
  const client = fakeClient(new FakeEngine(history));
  const { container } = renderRepoView(
    <GraphView client={client} repoId="repo-1" {...props} />,
    options,
  );
  return { grid: await screen.findByRole('grid'), container };
}

function rowOf(grid: HTMLElement, index: number): HTMLElement {
  const row = grid.querySelector<HTMLElement>(`[data-row-index="${String(index)}"]`);
  if (row === null) throw new Error(`no hay fila ${String(index)}`);
  return row;
}

describe('GraphView: filas', () => {
  it('pinta mensaje, autor, fecha y sha corto de cada commit', async () => {
    const row = graphRow('0123456789abcdef', {
      summary: 'feat(graph): carriles',
      authorName: 'Marta Rios',
      authorEmail: 'marta@example.com',
      isHead: true,
    });
    const { grid } = await mountGrid([row]);
    const cells = within(rowOf(grid, 0)).getAllByRole('gridcell');

    expect(cells).toHaveLength(6);
    expect(cells[2]).toHaveTextContent('feat(graph): carriles');
    expect(cells[2]?.querySelector('[data-head="true"]')).not.toBeNull();
    expect(cells[3]).toHaveTextContent('MRMarta Rios');
    expect(cells[3]).toHaveAttribute('title', 'marta@example.com');
    const time = within(cells[4]!).getByText(/^\d{1,2}\/\d{1,2} \d{2}:\d{2}$/u);
    expect(time.tagName).toBe('TIME');
    expect(time).toHaveAttribute('dateTime', '2026-04-12T10:24:00+02:00');
    expect(time.getAttribute('title')).toMatch(/2026/u);
    expect(cells[5]).toHaveTextContent('0123456');
    expect(rowOf(grid, 0)).toHaveAttribute('aria-rowindex', '2');
  });

  it('ensena dos pildoras de refs y resume el resto con "+n"', async () => {
    const refs = [
      refLabel('main', { isHead: true }),
      refLabel('origin/main', { kind: 'remoteBranch', fullName: 'refs/remotes/origin/main' }),
      refLabel('v1.0', { kind: 'tag', fullName: 'refs/tags/v1.0' }),
    ];
    const { grid } = await mountGrid([graphRow('a', { refs, lane: 2 })]);
    const cell = within(rowOf(grid, 0)).getAllByRole('gridcell')[0]!;

    expect(cell).toHaveTextContent('Rama actualmain');
    expect(cell).toHaveTextContent('Rama remotaorigin/main');
    expect(cell).toHaveTextContent('+11 referencia más: v1.0');
    expect(cell.querySelector('[data-head="true"]')).toHaveAttribute('title', 'refs/heads/main');
    // El tipo va en el DOM: VS Code esconde el glifo de las ramas locales por CSS.
    expect(cell.querySelector('[data-head="true"]')).toHaveAttribute('data-kind', 'localBranch');
    expect(cell.querySelector('[data-kind="remoteBranch"]')).toHaveTextContent('origin/main');
    expect(cell.querySelectorAll('svg')).toHaveLength(2);
  });

  it('dibuja aristas y nodos del tramo visible con las iniciales del autor', async () => {
    const history = [
      graphRow('m', {
        parents: ['a', 'b'],
        edges: [graphEdge('a', 1, { from: 0 }), graphEdge('b', 2, { from: 0, travel: 1 }, 'merge')],
      }),
      graphRow('a', { parents: ['b'], edges: [graphEdge('b', 2, { from: 0, to: 1 })] }),
      graphRow('b', { lane: 1, authorName: 'ezequiel' }),
    ];
    const { container } = await mountGrid(history);
    const svg = container.querySelector('[data-sha]')?.closest('svg');

    expect(svg?.querySelectorAll('path')).toHaveLength(3);
    expect(svg?.querySelector('path')).toHaveAttribute('d', 'M16 20 L16 60');
    expect(svg?.querySelector('[data-sha="m"] text')).toBeNull();
    expect(svg?.querySelector('[data-sha="b"] text')).toHaveTextContent('EZ');
    expect(svg).toHaveAttribute('viewBox', '0 0 130 120');
  });

  it('con densidad compacta usa filas de 28 px y las columnas de VS Code', async () => {
    const history = linearRows(3);
    history[0] = { ...history[0]!, refs: [refLabel('main')] };
    const { grid } = await mountGrid(history, { density: 'compact' }, { icons: codicons });

    expect(grid).toHaveAttribute('data-density', 'compact');
    expect(rowOf(grid, 0).querySelector('.codicon')).not.toBeNull();
    expect(rowOf(grid, 1).style.transform).toBe('translateY(28px)');
    expect(grid.style.getPropertyValue('--rv-columns')).toBe(
      '130px 120px minmax(150px, 1fr) 100px 84px 60px',
    );
  });

  it('traduce las cabeceras al ingles', async () => {
    const { grid } = await mountGrid(linearRows(1), {}, { locale: 'en' });

    expect(within(grid).getByRole('columnheader', { name: 'Author' })).toBeInTheDocument();
    expect(grid).toHaveAccessibleName('Commit history');
  });
});

describe('GraphView: seleccion', () => {
  it('selecciona con un clic, avisa al host y marca el nodo con su anillo', async () => {
    const onSelectionChange = vi.fn();
    const history = linearRows(5);
    const { grid, container } = await mountGrid(history, { onSelectionChange });

    await userEvent.click(within(rowOf(grid, 2)).getAllByRole('gridcell')[2]!);

    expect(rowOf(grid, 2)).toHaveAttribute('aria-selected', 'true');
    expect(onSelectionChange).toHaveBeenCalledWith(history[2]);
    expect(grid).toHaveAttribute('aria-activedescendant', rowOf(grid, 2).id);
    expect(container.querySelector('[data-sha="c2"][data-selected="true"] circle')).not.toBeNull();
  });

  it('se mueve con el teclado y activa con Intro y con doble clic', async () => {
    const onActivate = vi.fn();
    const history = linearRows(8);
    const { grid } = await mountGrid(history, { onActivate });

    grid.focus();
    await userEvent.keyboard('{ArrowDown}{ArrowDown}');
    expect(rowOf(grid, 1)).toHaveAttribute('aria-selected', 'true');

    await userEvent.keyboard('{End}');
    expect(rowOf(grid, 7)).toHaveAttribute('aria-selected', 'true');
    await userEvent.keyboard('{Home}{Enter}');
    expect(onActivate).toHaveBeenLastCalledWith(history[0]);

    await userEvent.dblClick(within(rowOf(grid, 3)).getAllByRole('gridcell')[5]!);
    expect(onActivate).toHaveBeenLastCalledWith(history[3]);
  });

  it('respeta la seleccion que controla el host', async () => {
    const onSelectionChange = vi.fn();
    const history = linearRows(4);
    const { grid } = await mountGrid(history, { selectedSha: 'c1', onSelectionChange });

    expect(rowOf(grid, 1)).toHaveAttribute('aria-selected', 'true');
    await userEvent.click(within(rowOf(grid, 3)).getAllByRole('gridcell')[2]!);

    expect(onSelectionChange).toHaveBeenCalledWith(history[3]);
    expect(rowOf(grid, 1)).toHaveAttribute('aria-selected', 'true');
    expect(rowOf(grid, 3)).toHaveAttribute('aria-selected', 'false');
  });

  it('ignora las teclas que no son de navegacion y los clics fuera de las filas', async () => {
    const onSelectionChange = vi.fn();
    const { grid } = await mountGrid(linearRows(3), { onSelectionChange });

    grid.focus();
    await userEvent.keyboard('a{Enter}');
    await userEvent.click(within(grid).getAllByRole('columnheader')[0]!);

    expect(onSelectionChange).not.toHaveBeenCalled();
    expect(grid).not.toHaveAttribute('aria-activedescendant');
  });
});
