import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import { renderWithProviders } from '../../test/render';
import { Chip } from '../chip/chip';

import { Badge, FileStatusBadge, PlatformBadge, PrStateBadge } from './badge';

describe('PlatformBadge', () => {
  it('pinta las iniciales con la marca como nombre accesible', () => {
    renderWithProviders(<PlatformBadge platform="gitlab" abbr="GL" name="GitLab" size={34} />);
    const badge = screen.getByRole('img', { name: 'GitLab' });
    expect(badge).toHaveTextContent('GL');
    expect(badge).toHaveAttribute('data-platform', 'gitlab');
    expect(badge).toHaveAttribute('data-size', '34');
  });

  it('un repo local lleva el icono de casa en lugar de iniciales', () => {
    renderWithProviders(<PlatformBadge platform="local" name="Solo local" />);
    const badge = screen.getByRole('img', { name: 'Solo local' });
    expect(badge.querySelector('svg')).not.toBeNull();
  });
});

describe('PrStateBadge', () => {
  it('traduce el estado con @git-rudder/i18n', () => {
    renderWithProviders(<PrStateBadge state="changes" />);
    expect(screen.getByText('Cambios solicitados')).toHaveAttribute('data-pr-state', 'changes');
  });

  it('en ingles usa el texto ingles', () => {
    renderWithProviders(<PrStateBadge state="merged" />, { locale: 'en' });
    expect(screen.getByText('Merged')).toBeInTheDocument();
  });
});

describe('FileStatusBadge', () => {
  it('pinta la letra con el nombre del estado como etiqueta', () => {
    renderWithProviders(<FileStatusBadge status="R" variant="box" />);
    const badge = screen.getByRole('img', { name: 'Renombrado' });
    expect(badge).toHaveTextContent('R');
    expect(badge).toHaveAttribute('data-variant', 'box');
  });
});

describe('Badge', () => {
  it('pinta el tono pedido', () => {
    renderWithProviders(<Badge tone="accent">Predeterminada</Badge>);
    expect(screen.getByText('Predeterminada')).toHaveAttribute('data-tone', 'accent');
  });
});

describe('Chip', () => {
  it('sin onChange es una etiqueta, no un boton', () => {
    renderWithProviders(<Chip tone="merged">Backend</Chip>);
    expect(screen.queryByRole('button')).toBeNull();
    expect(screen.getByText('Backend')).toBeInTheDocument();
  });

  it('con onChange es un filtro que se activa con el contador a la vista', async () => {
    const onChange = vi.fn();
    renderWithProviders(
      <Chip tone="info" count={3} onChange={onChange}>
        Abierto
      </Chip>,
    );
    const chip = screen.getByRole('button', { name: /Abierto/u });
    expect(chip).toHaveTextContent('3');
    expect(chip).toHaveAttribute('aria-pressed', 'false');
    await userEvent.click(chip);
    expect(onChange).toHaveBeenCalledWith(true);
  });
});
