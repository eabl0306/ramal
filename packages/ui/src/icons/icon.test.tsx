import { render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import { CODICONS, codicons } from './codicons';
import { Icon, IconProvider } from './icon';
import { ICON_NAMES } from './icon-names';
import { LUCIDE_ICONS, lucideIcons } from './lucide';

describe('juegos de iconos', () => {
  it('Lucide y Codicons cubren todos los nombres', () => {
    expect(Object.keys(LUCIDE_ICONS).sort()).toEqual([...ICON_NAMES].sort());
    expect(Object.keys(CODICONS).sort()).toEqual([...ICON_NAMES].sort());
    expect(Object.values(CODICONS).every((name) => name.startsWith('codicon-'))).toBe(true);
  });
});

describe('Icon', () => {
  it('con Lucide pinta un SVG decorativo oculto a los lectores de pantalla', () => {
    const { container } = render(
      <IconProvider icons={lucideIcons}>
        <Icon name="branch" />
      </IconProvider>,
    );
    const svg = container.querySelector('svg');
    expect(svg).not.toBeNull();
    expect(svg).toHaveAttribute('aria-hidden', 'true');
    expect(svg).toHaveAttribute('width', '16');
  });

  it('con nombre accesible se anuncia como imagen', () => {
    render(
      <IconProvider icons={lucideIcons}>
        <Icon name="conflict" label="Conflicto" size={14} />
      </IconProvider>,
    );
    expect(screen.getByRole('img', { name: 'Conflicto' })).toHaveAttribute('width', '14');
  });

  it('con Codicons pinta la clase del glifo', () => {
    const { container } = render(
      <IconProvider icons={codicons}>
        <Icon name="stash" className="extra" size={12} />
      </IconProvider>,
    );
    const glyph = container.querySelector('span');
    expect(glyph).toHaveClass('codicon', 'codicon-git-stash', 'extra');
    expect(glyph).toHaveStyle({ fontSize: '12px' });
  });

  it('sin provider falla con un mensaje claro', () => {
    vi.spyOn(console, 'error').mockImplementation(() => undefined);
    expect(() => render(<Icon name="add" />)).toThrow(/IconProvider/u);
  });
});
