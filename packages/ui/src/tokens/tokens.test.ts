import { describe, expect, it } from 'vitest';

import { DENSITIES, DENSITY_NAMES } from './densities';
import { cssVar, laneColor } from './tokens';

describe('tokens', () => {
  it('cssVar envuelve el token en var()', () => {
    expect(cssVar('--r-accent')).toBe('var(--r-accent)');
  });

  it('el color de carril da la vuelta a la paleta a partir del sexto', () => {
    expect(laneColor(0)).toBe('var(--r-lane-0)');
    expect(laneColor(5)).toBe('var(--r-lane-5)');
    expect(laneColor(6)).toBe('var(--r-lane-0)');
    expect(laneColor(13)).toBe('var(--r-lane-1)');
    expect(laneColor(-1)).toBe('var(--r-lane-5)');
  });
});

describe('DENSITIES', () => {
  it('tiene las dos densidades con la geometria del grafo de cada prototipo', () => {
    expect(DENSITY_NAMES).toEqual(['comfortable', 'compact']);
    expect(DENSITIES.comfortable.graph).toMatchObject({ originX: 16, controlNear: 22 });
    expect(DENSITIES.compact.graph).toMatchObject({ originX: 14, controlNear: 16 });
  });
});
