import { fireEvent, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { type ReactElement, useState } from 'react';
import { describe, expect, it, vi } from 'vitest';

import { renderWithProviders } from '../../test/render';
import { IconButton } from '../icon-button/icon-button';

import { ContextMenu } from './context-menu';
import type { MenuEntry } from './menu-model';
import { useContextMenu } from './use-context-menu';

interface HarnessProps {
  readonly onCheckout?: () => void;
  readonly onDelete?: () => void;
  readonly onMove?: (workspace: string) => void;
}

/** Una fila con menu contextual y boton ⋯, como la tarjeta de un repo. */
function Harness({ onCheckout, onDelete, onMove }: HarnessProps): ReactElement {
  const menu = useContextMenu();
  const [workspaces, setWorkspaces] = useState<readonly string[]>(['w1']);
  const toggle = (id: string): void => {
    onMove?.(id);
    setWorkspaces((current) =>
      current.includes(id) ? current.filter((item) => item !== id) : [...current, id],
    );
  };
  const entries: MenuEntry[] = [
    {
      id: 'checkout',
      label: 'Checkout',
      hint: 'Ctrl+Enter',
      icon: 'branch',
      ...withAction(onCheckout),
    },
    {
      id: 'workspaces',
      label: 'Workspaces',
      children: ['w1', 'w2'].map((id) => ({
        id,
        label: id === 'w1' ? 'Producto' : 'Backend',
        checked: workspaces.includes(id),
        keepOpen: true,
        onAction: () => {
          toggle(id);
        },
      })),
    },
    { kind: 'separator', id: 'sep' },
    { id: 'delete', label: 'Eliminar rama', danger: true, ...withAction(onDelete) },
    { id: 'disabled', label: 'Blame', disabled: true },
  ];
  return (
    <>
      <div data-testid="row" onContextMenu={menu.openAtPointer}>
        repo
      </div>
      <IconButton
        icon="more"
        label="Mas acciones"
        onPress={(event) => {
          menu.openAtElement(event.target);
        }}
      />
      <ContextMenu
        entries={entries}
        anchor={menu.anchor}
        onClose={menu.close}
        label="Repositorio"
      />
    </>
  );
}

function withAction(action: (() => void) | undefined): { onAction?: () => void } {
  return action === undefined ? {} : { onAction: action };
}

function openByRightClick(): HTMLElement {
  fireEvent.contextMenu(screen.getByTestId('row'), { clientX: 120, clientY: 80 });
  return screen.getByRole('menu', { name: 'Repositorio' });
}

describe('ContextMenu', () => {
  it('se abre con clic derecho, ejecuta la accion y se cierra', async () => {
    const onCheckout = vi.fn();
    renderWithProviders(<Harness onCheckout={onCheckout} />);
    const menu = openByRightClick();
    const item = within(menu).getByRole('menuitem', { name: /Checkout/u });
    expect(item).toHaveTextContent('Ctrl+Enter');
    await userEvent.click(item);
    expect(onCheckout).toHaveBeenCalledTimes(1);
    expect(screen.queryByRole('menu')).toBeNull();
  });

  it('marca las acciones de peligro y respeta las deshabilitadas', async () => {
    const onDelete = vi.fn();
    renderWithProviders(<Harness onDelete={onDelete} />);
    const menu = openByRightClick();
    const disabled = within(menu).getByRole('menuitem', { name: 'Blame' });
    expect(disabled).toHaveAttribute('aria-disabled', 'true');
    const danger = within(menu).getByRole('menuitem', { name: 'Eliminar rama' });
    expect(danger).toHaveAttribute('data-danger', 'true');
    expect(within(menu).getByRole('separator')).toBeInTheDocument();
    await userEvent.click(danger);
    expect(onDelete).toHaveBeenCalledTimes(1);
  });

  it('Esc lo cierra', async () => {
    renderWithProviders(<Harness />);
    openByRightClick();
    await userEvent.keyboard('{Escape}');
    expect(screen.queryByRole('menu')).toBeNull();
  });

  it('el boton de mas acciones lo abre pegado al boton', async () => {
    renderWithProviders(<Harness />);
    await userEvent.click(screen.getByRole('button', { name: 'Mas acciones' }));
    expect(screen.getByRole('menu', { name: 'Repositorio' })).toBeInTheDocument();
  });

  it('el submenu tiene casillas que dejan el menu abierto', async () => {
    const onMove = vi.fn();
    renderWithProviders(<Harness onMove={onMove} />);
    const menu = openByRightClick();
    await userEvent.click(within(menu).getByRole('menuitem', { name: 'Workspaces' }));
    const submenu = screen.getByRole('menu', { name: 'Workspaces' });
    const product = within(submenu).getByRole('menuitemcheckbox', { name: 'Producto' });
    const backend = within(submenu).getByRole('menuitemcheckbox', { name: 'Backend' });
    expect(product).toHaveAttribute('aria-checked', 'true');
    expect(backend).toHaveAttribute('aria-checked', 'false');
    await userEvent.click(backend);
    expect(onMove).toHaveBeenCalledWith('w2');
    expect(screen.getByRole('menu', { name: 'Workspaces' })).toBeInTheDocument();
    expect(
      within(screen.getByRole('menu', { name: 'Workspaces' })).getByRole('menuitemcheckbox', {
        name: 'Backend',
      }),
    ).toHaveAttribute('aria-checked', 'true');
  });

  it('la tecla de menu sin coordenadas lo abre pegado a la fila', () => {
    renderWithProviders(<Harness />);
    fireEvent.contextMenu(screen.getByTestId('row'), { clientX: 0, clientY: 0 });
    expect(screen.getByRole('menu', { name: 'Repositorio' })).toBeInTheDocument();
  });
});
