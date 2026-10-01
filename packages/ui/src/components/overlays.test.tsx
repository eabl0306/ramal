import { act, render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import type { ReactElement } from 'react';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { renderWithProviders } from '../test/render';

import { Modal } from './modal/modal';
import { useToast } from './toast/toast-context';
import { TOAST_DURATION, ToastProvider } from './toast/toast-provider';

describe('Modal', () => {
  it('abierto es un dialogo con titulo, cuerpo y pie', () => {
    renderWithProviders(
      <Modal
        isOpen
        onOpenChange={vi.fn()}
        title="Crear rama"
        subtitle="Desde HEAD"
        footer={<button type="button">Crear</button>}
      >
        <p>Cuerpo</p>
      </Modal>,
    );
    const dialog = screen.getByRole('dialog', { name: 'Crear rama' });
    expect(dialog).toHaveTextContent('Desde HEAD');
    expect(dialog).toHaveTextContent('Cuerpo');
    expect(screen.getByRole('button', { name: 'Crear' })).toBeInTheDocument();
  });

  it('Esc pide cerrarlo', async () => {
    const onOpenChange = vi.fn();
    renderWithProviders(
      <Modal isOpen onOpenChange={onOpenChange} title="Descartar" role="alertdialog" size="md">
        <p>Seguro</p>
      </Modal>,
    );
    expect(screen.getByRole('alertdialog', { name: 'Descartar' })).toBeInTheDocument();
    await userEvent.keyboard('{Escape}');
    expect(onOpenChange).toHaveBeenCalledWith(false);
  });

  it('cerrado no pinta nada', () => {
    renderWithProviders(
      <Modal isOpen={false} onOpenChange={vi.fn()} title="Oculto">
        <p>Nada</p>
      </Modal>,
    );
    expect(screen.queryByRole('dialog')).toBeNull();
  });
});

function ToastButtons(): ReactElement {
  const toast = useToast();
  return (
    <>
      <button type="button" onClick={() => toast.show('Fetch completado')}>
        ok
      </button>
      <button
        type="button"
        onClick={() => {
          const id = toast.show('Sin conexion', 'error');
          toast.dismiss(id);
        }}
      >
        quitar
      </button>
    </>
  );
}

describe('ToastProvider', () => {
  afterEach(() => {
    vi.useRealTimers();
  });

  it('ensena el aviso en una region viva y lo quita pasado el tiempo', () => {
    vi.useFakeTimers();
    renderWithProviders(
      <ToastProvider>
        <ToastButtons />
      </ToastProvider>,
    );
    expect(screen.getByRole('region', { name: 'Notificaciones' })).toBeInTheDocument();
    act(() => {
      screen.getByRole('button', { name: 'ok' }).click();
    });
    expect(screen.getByRole('status')).toHaveTextContent('Fetch completado');
    expect(screen.getByRole('status')).toHaveAttribute('data-kind', 'ok');
    act(() => {
      vi.advanceTimersByTime(TOAST_DURATION);
    });
    expect(screen.queryByRole('status')).toBeNull();
  });

  it('se puede quitar un aviso antes de tiempo', () => {
    renderWithProviders(
      <ToastProvider duration={60_000}>
        <ToastButtons />
      </ToastProvider>,
    );
    act(() => {
      screen.getByRole('button', { name: 'quitar' }).click();
    });
    expect(screen.queryByRole('status')).toBeNull();
  });

  it('useToast sin provider falla con un mensaje claro', () => {
    vi.spyOn(console, 'error').mockImplementation(() => undefined);
    expect(() => render(<ToastButtons />)).toThrow(/ToastProvider/u);
  });
});
