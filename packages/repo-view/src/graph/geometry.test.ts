import { describe, expect, it } from 'vitest';

import { COMFORTABLE, COMPACT } from '../test/fixtures';

import { COLUMN_LAYOUTS, gridTemplate } from './columns';
import { graphColumnWidth, laneX, rowY } from './geometry';

describe('laneX y rowY', () => {
  it('siguen X = 16 + carril * 20 e Y = fila * 40 + 20 en escritorio', () => {
    expect([0, 1, 5].map((lane) => laneX(lane, COMFORTABLE))).toEqual([16, 36, 116]);
    expect([0, 1, 10].map((row) => rowY(row, COMFORTABLE))).toEqual([20, 60, 420]);
  });

  it('siguen X = 14 + carril * 20 e Y = fila * 28 + 14 en VS Code', () => {
    expect(laneX(2, COMPACT)).toBe(54);
    expect(rowY(2, COMPACT)).toBe(70);
  });
});

describe('graphColumnWidth', () => {
  it('se queda en el ancho del diseno mientras caben los carriles', () => {
    expect(graphColumnWidth(0, COMFORTABLE, 130)).toBe(130);
    expect(graphColumnWidth(6, COMFORTABLE, 130)).toBe(130);
  });

  it('crece con mas carriles para que el ultimo nodo y su anillo quepan', () => {
    // Carril 7: 16 + 7 * 20 = 156, mas radio 9, anillo 4 y medio trazo.
    expect(graphColumnWidth(8, COMFORTABLE, 130)).toBe(170);
    expect(graphColumnWidth(8, COMPACT, 120)).toBe(165);
  });
});

describe('columnas de la tabla', () => {
  it('escritorio usa 200 | grafo | 1fr | 150 | 110 | 76', () => {
    expect(gridTemplate(COLUMN_LAYOUTS.comfortable, 130)).toBe(
      '200px 130px minmax(0px, 1fr) 150px 110px 76px',
    );
  });

  it('VS Code usa las columnas estrechas de su prototipo', () => {
    expect(gridTemplate(COLUMN_LAYOUTS.compact, 140)).toBe(
      '130px 140px minmax(150px, 1fr) 100px 84px 60px',
    );
  });
});
