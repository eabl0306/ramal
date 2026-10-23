import '@testing-library/jest-dom/vitest';

import { cleanup } from '@testing-library/react';
import { afterEach, beforeEach } from 'vitest';

/**
 * jsdom no maqueta: todo mide 0 px y el virtualizador del grafo no pintaria
 * ninguna fila. Se dan medidas fijas a los elementos, las que lee
 * @tanstack/virtual-core (clientWidth y clientHeight para el contenedor con
 * scroll, offsetWidth y offsetHeight para medir cada fila), para que los tests
 * vean las filas de la primera pantalla.
 */
const VIEWPORT = { height: 400, width: 1000 };

const SIZES: Readonly<Record<string, number>> = {
  offsetHeight: VIEWPORT.height,
  clientHeight: VIEWPORT.height,
  offsetWidth: VIEWPORT.width,
  clientWidth: VIEWPORT.width,
};

const originals = Object.keys(SIZES).map(
  (name) => [name, Object.getOwnPropertyDescriptor(HTMLElement.prototype, name)] as const,
);

beforeEach(() => {
  for (const [name, size] of Object.entries(SIZES)) {
    Object.defineProperty(HTMLElement.prototype, name, { configurable: true, get: () => size });
  }
});

// Vitest corre sin globals: el limpiado de Testing Library se declara aqui, y
// despues se devuelven las medidas originales de jsdom.
afterEach(() => {
  cleanup();
  for (const [name, original] of originals) {
    if (original !== undefined) Object.defineProperty(HTMLElement.prototype, name, original);
  }
});
