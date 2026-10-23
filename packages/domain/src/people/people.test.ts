import { describe, expect, it } from 'vitest';

import { avatarColorIndex } from './avatar-color';
import { authorInitials } from './initials';

describe('authorInitials', () => {
  it('toma la primera letra de la primera y de la ultima palabra', () => {
    expect(authorInitials('Ana Maria Ruiz')).toBe('AR');
    expect(authorInitials('  marta   rios ')).toBe('MR');
  });

  it('con una sola palabra toma sus dos primeras letras', () => {
    expect(authorInitials('ezequiel')).toBe('EZ');
    expect(authorInitials('j')).toBe('J');
    expect(authorInitials('dev.bot')).toBe('DB');
  });

  it('no parte los caracteres fuera del plano basico ni pierde los acentos', () => {
    expect(authorInitials('Ångström Ñu')).toBe('ÅÑ');
    expect(authorInitials('𝒜lex')).toBe('𝒜L');
  });

  it('da ? cuando no hay ninguna letra ni numero', () => {
    expect(authorInitials('  -- ')).toBe('?');
    expect(authorInitials('')).toBe('?');
  });
});

describe('avatarColorIndex', () => {
  const PALETTE = 6;

  it('el color sale del modulo del tamano de la paleta', () => {
    for (const name of ['Ana Ruiz', 'marta', 'ezequiel@example.com', 'Ångström', '𝒜lex']) {
      const index = avatarColorIndex(name, PALETTE);
      expect(Number.isInteger(index)).toBe(true);
      expect(index).toBeGreaterThanOrEqual(0);
      expect(index).toBeLessThan(PALETTE);
    }
  });

  it('el mismo nombre da siempre el mismo color', () => {
    expect(avatarColorIndex('Ana Ruiz', PALETTE)).toBe(avatarColorIndex('Ana Ruiz', PALETTE));
  });

  it('no depende de los espacios de los lados ni de las mayusculas', () => {
    const base = avatarColorIndex('Ana Ruiz', PALETTE);
    expect(avatarColorIndex('  Ana Ruiz  ', PALETTE)).toBe(base);
    expect(avatarColorIndex('ANA RUIZ', PALETTE)).toBe(base);
  });

  it('nombres parecidos no caen siempre en el mismo color', () => {
    const colors = new Set(
      ['Ana Ruiz', 'Ana Rubio', 'Ana Ruaz', 'Ana Ruia'].map((name) =>
        avatarColorIndex(name, PALETTE),
      ),
    );
    expect(colors.size).toBeGreaterThan(1);
  });

  it('reparte los nombres por toda la paleta', () => {
    const names = Array.from({ length: 60 }, (_unused, index) => `Persona ${String(index)}`);
    const used = new Set(names.map((name) => avatarColorIndex(name, PALETTE)));
    expect(used.size).toBe(PALETTE);
  });

  it('un nombre vacio o una paleta vacia dan 0, que es un color valido', () => {
    expect(avatarColorIndex('', PALETTE)).toBe(0);
    expect(avatarColorIndex('   ', PALETTE)).toBe(0);
    expect(avatarColorIndex('Ana Ruiz', 0)).toBe(0);
    expect(avatarColorIndex('Ana Ruiz', -3)).toBe(0);
  });
});
