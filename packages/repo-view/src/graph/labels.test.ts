import { describe, expect, it } from 'vitest';

import { refLabel } from '../test/fixtures';

import { nextRowIndex } from './keyboard';
import { MAX_VISIBLE_REFS, refColor, refIcon, splitRefs } from './refs';

describe('refs de una fila', () => {
  const head = refLabel('main', { isHead: true });
  const remote = refLabel('origin/main', { kind: 'remoteBranch' });
  const tag = refLabel('v1.0', { kind: 'tag', fullName: 'refs/tags/v1.0' });
  const local = refLabel('feature/graph');

  it('ensena dos pildoras y deja el resto para "+n"', () => {
    expect(MAX_VISIBLE_REFS).toBe(2);
    expect(splitRefs([head, remote, tag, local])).toEqual({
      shown: [head, remote],
      hidden: [tag, local],
    });
    expect(splitRefs([head])).toEqual({ shown: [head], hidden: [] });
  });

  it('elige el icono semantico de cada clase de ref', () => {
    expect([head, remote, tag, local].map(refIcon)).toEqual(['check', 'remote', 'tag', 'branch']);
  });

  it('pinta los tags en el tono de aviso y las ramas en el color del carril', () => {
    expect(refColor(tag, 3)).toBe('var(--r-warn)');
    expect(refColor(local, 3)).toBe('var(--r-lane-3)');
    expect(refColor(remote, 7)).toBe('var(--r-lane-1)');
  });
});

describe('nextRowIndex', () => {
  it('baja, sube y se queda dentro de la tabla', () => {
    expect(nextRowIndex('ArrowDown', 3, 10, 5)).toBe(4);
    expect(nextRowIndex('ArrowDown', 9, 10, 5)).toBe(9);
    expect(nextRowIndex('ArrowUp', 3, 10, 5)).toBe(2);
    expect(nextRowIndex('ArrowUp', 0, 10, 5)).toBe(0);
  });

  it('salta una pagina y va a los extremos', () => {
    expect(nextRowIndex('PageDown', 3, 10, 5)).toBe(8);
    expect(nextRowIndex('PageDown', 8, 10, 5)).toBe(9);
    expect(nextRowIndex('PageUp', 8, 10, 5)).toBe(3);
    expect(nextRowIndex('PageUp', 2, 10, 5)).toBe(0);
    expect(nextRowIndex('Home', 6, 10, 5)).toBe(0);
    expect(nextRowIndex('End', 1, 10, 5)).toBe(9);
  });

  it('sin seleccion empieza por arriba al bajar y por abajo al subir', () => {
    expect(nextRowIndex('ArrowDown', null, 10, 5)).toBe(0);
    expect(nextRowIndex('ArrowUp', null, 10, 5)).toBe(9);
    expect(nextRowIndex('PageDown', null, 10, 5)).toBe(4);
    expect(nextRowIndex('PageUp', null, 10, 5)).toBe(5);
  });

  it('una pagina nunca es menor que una fila', () => {
    expect(nextRowIndex('PageDown', 3, 10, 0)).toBe(4);
  });

  it('ignora otras teclas y las tablas vacias', () => {
    expect(nextRowIndex('a', 3, 10, 5)).toBeNull();
    expect(nextRowIndex('toString', 3, 10, 5)).toBeNull();
    expect(nextRowIndex('ArrowDown', null, 0, 5)).toBeNull();
  });
});
