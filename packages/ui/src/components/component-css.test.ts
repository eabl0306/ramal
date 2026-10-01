import { describe, expect, it } from 'vitest';

import { readSourceCss, ruleOf } from '../test/css';

/**
 * Medidas de los prototipos *.dc.html que jsdom no puede ver, porque no aplica
 * los CSS Modules: se leen de la hoja. Cada valor es el del diseno, que manda
 * sobre el README del handoff.
 */

const button = readSourceCss('components/button/button.module.css');
const textField = readSourceCss('components/text-field/text-field.module.css');
const segmented = readSourceCss('components/segmented-control/segmented-control.module.css');

describe('button.module.css', () => {
  it('los botones de 42 de la web llevan esquinas de 10 y relleno de 18', () => {
    const rule = ruleOf(button, ".button[data-size='42']");
    expect(rule.get('height')).toBe('42px');
    expect(rule.get('padding')).toBe('0 18px');
    expect(rule.get('border-radius')).toBe('10px');
  });

  it('el envio de entrar y crear cuenta mide 44 con letra de 15', () => {
    const rule = ruleOf(button, ".button[data-size='44']");
    expect(rule.get('height')).toBe('44px');
    expect(rule.get('border-radius')).toBe('10px');
    expect(rule.get('font-size')).toBe('15px');
  });

  it('el de 48 de la portada tiene el relleno de 22 del prototipo', () => {
    expect(ruleOf(button, ".button[data-size='48']").get('padding')).toBe('0 22px');
  });

  it('el proveedor va sobre el fondo de la pagina y al pasar solo cambia el borde', () => {
    expect(ruleOf(button, ".button[data-variant='outline']").get('background')).toBe('var(--r-bg)');
    const hovered = ruleOf(button, ".button[data-variant='outline'][data-hovered]");
    expect(hovered.get('border-color')).toBe('var(--r-accent)');
    expect(hovered.get('background')).toBe('var(--r-bg)');
  });
});

describe('text-field.module.css', () => {
  it('la caja de 42 de la tarjeta de entrar lleva esquinas de 10', () => {
    const rule = ruleOf(textField, ".field[data-size='42'] .input");
    expect(rule.get('height')).toBe('42px');
    expect(rule.get('border-radius')).toBe('10px');
  });
});

describe('segmented-control.module.css', () => {
  it('la barra y la cabecera web pegan los segmentos en un contenedor de esquinas de 8', () => {
    for (const variant of ['bar', 'nav']) {
      const rule = ruleOf(segmented, `.group[data-variant='${variant}']`);
      expect(rule.get('gap'), variant).toBe('0');
      expect(rule.get('padding'), variant).toBe('2px');
      expect(rule.get('border-radius'), variant).toBe('8px');
    }
  });

  it('los segmentos de solo icono miden 30 de ancho en las dos', () => {
    for (const variant of ['bar', 'nav']) {
      const rule = ruleOf(segmented, `.group[data-variant='${variant}'] .segment[data-icon-only]`);
      expect(rule.get('width'), variant).toBe('30px');
      expect(rule.get('padding'), variant).toBe('0');
    }
  });

  it('la barra mide 24 sobre el fondo y la cabecera web 26 sin fondo', () => {
    expect(ruleOf(segmented, ".group[data-variant='bar'] .segment").get('height')).toBe('24px');
    expect(ruleOf(segmented, ".group[data-variant='nav'] .segment").get('height')).toBe('26px');
    expect(ruleOf(segmented, ".group[data-variant='nav']").get('background')).toBe('transparent');
  });
});
