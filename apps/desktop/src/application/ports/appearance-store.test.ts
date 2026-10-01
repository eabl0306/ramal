import { describe, expect, it } from 'vitest';

import { THEMES, isThemeName } from './appearance-store';

describe('isThemeName', () => {
  it('acepta los dos temas de ramal.css', () => {
    expect(THEMES.every(isThemeName)).toBe(true);
  });

  it.each(['system', 'Dark', '', null, 1])('rechaza %j', (value) => {
    expect(isThemeName(value)).toBe(false);
  });
});
