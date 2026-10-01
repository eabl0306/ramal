import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import { renderWithProviders } from '../test/render';

import { SegmentedControl } from './segmented-control/segmented-control';
import { TextField } from './text-field/text-field';
import { Toggle } from './toggle/toggle';

describe('Toggle', () => {
  it('es un interruptor con nombre accesible que avisa al cambiar', async () => {
    const onChange = vi.fn();
    renderWithProviders(<Toggle isSelected={false} onChange={onChange} label="Fetch automatico" />);
    const toggle = screen.getByRole('switch', { name: 'Fetch automatico' });
    expect(toggle).not.toBeChecked();
    await userEvent.click(toggle);
    expect(onChange).toHaveBeenCalledWith(true);
  });

  it('con texto visible lo usa como etiqueta', () => {
    renderWithProviders(
      <Toggle isSelected onChange={vi.fn()}>
        Firmar commits
      </Toggle>,
    );
    expect(screen.getByRole('switch', { name: 'Firmar commits' })).toBeChecked();
  });
});

describe('SegmentedControl', () => {
  const options = [
    { id: 'split', label: 'Lado a lado' },
    { id: 'unified', label: 'Unificada' },
  ] as const;

  it('marca la opcion actual y avisa al elegir otra', async () => {
    const onChange = vi.fn();
    renderWithProviders(
      <SegmentedControl
        label="Modo del diff"
        options={options}
        value="split"
        onChange={onChange}
        variant="surface"
      />,
    );
    expect(screen.getByRole('radiogroup', { name: 'Modo del diff' })).toHaveAttribute(
      'data-variant',
      'surface',
    );
    expect(screen.getByRole('radio', { name: 'Lado a lado' })).toBeChecked();
    await userEvent.click(screen.getByRole('radio', { name: 'Unificada' }));
    expect(onChange).toHaveBeenCalledWith('unified');
  });

  it('una opcion de solo icono usa su texto como nombre accesible', () => {
    renderWithProviders(
      <SegmentedControl
        label="Tema"
        variant="bar"
        options={[
          { id: 'light', label: 'Claro', icon: 'theme-light', iconOnly: true },
          { id: 'dark', label: 'Oscuro', icon: 'theme-dark', iconOnly: true },
        ]}
        value="dark"
        onChange={vi.fn()}
      />,
    );
    expect(screen.getByRole('radio', { name: 'Oscuro' })).toBeChecked();
    expect(screen.getByRole('radio', { name: 'Claro' })).not.toHaveTextContent('Claro');
  });
});

describe('TextField', () => {
  it('asocia la etiqueta y avisa de cada cambio', async () => {
    const onChange = vi.fn();
    renderWithProviders(
      <TextField
        label="Nombre de la rama"
        value=""
        onChange={onChange}
        placeholder="feature/x"
        mono
      />,
    );
    const input = screen.getByLabelText('Nombre de la rama');
    expect(input).toHaveAttribute('placeholder', 'feature/x');
    await userEvent.type(input, 'a');
    expect(onChange).toHaveBeenCalledWith('a');
  });

  it('ensena el error en linea y marca el campo como invalido', () => {
    renderWithProviders(
      <TextField
        label="Email"
        type="email"
        value="x"
        onChange={vi.fn()}
        description="El de tu cuenta"
        errorMessage="Escribe un email valido."
      />,
    );
    const input = screen.getByLabelText('Email');
    expect(input).toHaveAttribute('aria-invalid', 'true');
    expect(input).toHaveAttribute('type', 'email');
    expect(screen.getByText('Escribe un email valido.')).toBeInTheDocument();
    expect(screen.getByText('El de tu cuenta')).toBeInTheDocument();
  });

  it('con filas es un area de texto', () => {
    renderWithProviders(<TextField label="Descripcion" value="" onChange={vi.fn()} rows={3} />);
    expect(screen.getByLabelText('Descripcion').tagName).toBe('TEXTAREA');
  });
});
