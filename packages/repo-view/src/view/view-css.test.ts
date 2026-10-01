import { describe, expect, it } from 'vitest';

import { readViewCss, ruleOf } from '../test/css';

const grid = readViewCss('graph-grid.module.css');
const pills = readViewCss('ref-pills.module.css');

describe('graph-grid.module.css', () => {
  it('en alto contraste la fila marca el paso con borde discontinuo y la elegida con continuo', () => {
    // Los temas de alto contraste dejan transparentes los fondos de hover y seleccion.
    const hover = ruleOf(grid, '.row:hover');
    expect(hover.get('outline')).toBe('1px dashed var(--r-hc-active-border)');
    expect(hover.get('outline-offset')).toBe('-1px');
    const selected = ruleOf(grid, ".row[aria-selected='true']");
    expect(selected.get('outline')).toBe('1px solid var(--r-hc-active-border)');
    expect(selected.get('outline-offset')).toBe('-1px');
  });

  it('la elegida usa la seleccion inactiva sin foco y la activa, con su tinta, con foco', () => {
    expect(ruleOf(grid, ".row[aria-selected='true']").get('background')).toBe(
      'var(--r-accent-soft)',
    );
    const focused = ruleOf(grid, ".grid:focus-within .row[aria-selected='true']");
    expect(focused.get('background')).toBe('var(--r-selection)');
    expect(focused.get('color')).toBe('var(--r-selection-ink)');
  });
});

describe('ref-pills.module.css', () => {
  it('en escritorio tine al 14% y 32% la rama de HEAD, con letra de 500', () => {
    expect(ruleOf(pills, '.ref').get('font-weight')).toBe('500');
    expect(ruleOf(pills, '.ref::before').get('opacity')).toBe('0.14');
    expect(ruleOf(pills, ".ref[data-head='true']::before").get('opacity')).toBe('0.32');
  });

  it('en VS Code tine al 12% y 30%, con letra normal y "+n" sobre la seleccion inactiva', () => {
    expect(ruleOf(pills, "[data-density='compact'] .ref").get('font-weight')).toBe('normal');
    expect(ruleOf(pills, "[data-density='compact'] .ref::before").get('opacity')).toBe('0.12');
    expect(
      ruleOf(pills, "[data-density='compact'] .ref[data-head='true']::before").get('opacity'),
    ).toBe('0.3');
    expect(ruleOf(pills, "[data-density='compact'] .more").get('background')).toBe(
      'var(--r-accent-soft)',
    );
  });

  it('en VS Code las ramas locales que no son HEAD van sin glifo', () => {
    const selector =
      "[data-density='compact'] .ref[data-kind='localBranch']:not([data-head='true']) .ref-icon";
    expect(ruleOf(pills, selector).get('display')).toBe('none');
  });
});
