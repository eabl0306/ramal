import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import { renderWithProviders } from '../../test/render';
import { IconButton } from '../icon-button/icon-button';

import { Button } from './button';

describe('Button', () => {
  it('es secundario de 36 px por defecto y avisa al pulsar', async () => {
    const onPress = vi.fn();
    renderWithProviders(<Button onPress={onPress}>Abrir</Button>);
    const button = screen.getByRole('button', { name: 'Abrir' });
    expect(button).toHaveAttribute('data-variant', 'secondary');
    expect(button).toHaveAttribute('data-size', '36');
    await userEvent.click(button);
    expect(onPress).toHaveBeenCalledTimes(1);
  });

  it('pinta la variante, el tamano, el bloque y el icono pedidos', () => {
    const { container } = renderWithProviders(
      <Button variant="primary" size={38} block icon="commit">
        Commit
      </Button>,
    );
    const button = screen.getByRole('button', { name: 'Commit' });
    expect(button).toHaveAttribute('data-variant', 'primary');
    expect(button).toHaveAttribute('data-size', '38');
    expect(button).toHaveAttribute('data-block', 'true');
    expect(container.querySelector('svg')).toHaveAttribute('width', '16');
  });

  it('deshabilitado no avisa', async () => {
    const onPress = vi.fn();
    renderWithProviders(
      <Button variant="danger" isDisabled onPress={onPress}>
        Eliminar
      </Button>,
    );
    await userEvent.click(screen.getByRole('button', { name: 'Eliminar' }));
    expect(onPress).not.toHaveBeenCalled();
  });
});

describe('IconButton', () => {
  it('usa la etiqueta como nombre accesible', async () => {
    const onPress = vi.fn();
    renderWithProviders(
      <IconButton
        icon="more"
        label="Mas acciones"
        variant="bordered"
        size={36}
        onPress={onPress}
      />,
    );
    const button = screen.getByRole('button', { name: 'Mas acciones' });
    expect(button).toHaveAttribute('data-variant', 'bordered');
    expect(button).toHaveAttribute('data-size', '36');
    await userEvent.click(button);
    expect(onPress).toHaveBeenCalledTimes(1);
  });
});
