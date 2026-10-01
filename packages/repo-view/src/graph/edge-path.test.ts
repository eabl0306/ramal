import { describe, expect, it } from 'vitest';

import { COMFORTABLE, COMPACT } from '../test/fixtures';

import { type EdgeSpan, edgePath } from './edge-path';

function span(
  childRow: number,
  parentRow: number | null,
  fromLane: number,
  travelLane: number,
  toLane: number,
): EdgeSpan {
  return { childRow, parentRow, fromLane, travelLane, toLane };
}

const BOTTOM = 400;

describe('edgePath con la geometria de escritorio', () => {
  // Los tres caminos esperados de data-and-algorithms 3.5 (M -> F, F -> E, E -> B).
  it('dibuja un merge con la curva en la primera fila y despues la vertical', () => {
    expect(edgePath(span(0, 1, 0, 1, 1), COMFORTABLE, BOTTOM)).toBe(
      'M16 20 C16 42 36 38 36 60 L36 60',
    );
  });

  it('dibuja una recta cuando hijo y padre estan en el mismo carril', () => {
    expect(edgePath(span(1, 3, 1, 1, 1), COMFORTABLE, BOTTOM)).toBe('M36 60 L36 140');
  });

  it('dibuja una rama que sale con la vertical y la curva en la ultima fila', () => {
    expect(edgePath(span(3, 5, 1, 1, 0), COMFORTABLE, BOTTOM)).toBe(
      'M36 140 L36 180 C36 202 16 198 16 220',
    );
  });

  it('enruta por el carril de viaje con curva, vertical y curva', () => {
    expect(edgePath(span(0, 4, 0, 2, 1), COMFORTABLE, BOTTOM)).toBe(
      'M16 20 C16 42 56 38 56 60 L56 140 C56 162 36 158 36 180',
    );
  });

  it('con el padre en la fila siguiente hace una sola curva al carril del padre', () => {
    expect(edgePath(span(2, 3, 0, 2, 1), COMFORTABLE, BOTTOM)).toBe(
      'M16 100 C16 122 36 118 36 140 L36 140',
    );
  });

  it('lleva hasta el borde inferior una arista cuyo padre no esta cargado', () => {
    expect(edgePath(span(8, null, 1, 1, 1), COMFORTABLE, BOTTOM)).toBe('M36 340 L36 400');
  });

  it('un merge sin padre cargado curva al carril de viaje y baja hasta el borde', () => {
    expect(edgePath(span(8, null, 0, 2, 2), COMFORTABLE, BOTTOM)).toBe(
      'M16 340 C16 362 56 358 56 380 L56 400',
    );
  });
});

describe('edgePath con la geometria de VS Code', () => {
  it('usa filas de 28 px, origen 14 y puntos de control 16 y 12', () => {
    expect(edgePath(span(0, 1, 0, 1, 1), COMPACT, BOTTOM)).toBe('M14 14 C14 30 34 26 34 42 L34 42');
    expect(edgePath(span(3, 5, 1, 1, 0), COMPACT, BOTTOM)).toBe(
      'M34 98 L34 126 C34 142 14 138 14 154',
    );
  });
});
