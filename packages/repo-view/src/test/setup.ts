import '@testing-library/jest-dom/vitest';

import { cleanup } from '@testing-library/react';
import { afterEach, beforeEach } from 'vitest';

/**
 * jsdom no maqueta: todo mide 0 px y el virtualizador no pintaria ninguna fila.
 * Se dan medidas fijas a los elementos, las que lee @tanstack/virtual-core: el
 * contenedor con scroll por clientWidth y clientHeight, y cada fila medida por
 * offsetWidth y offsetHeight. Hasta virtual-core 3.19 el rect del contenedor
 * salia tambien de offset*, asi que con solo esas dos la tabla se quedaba vacia.
 * VIEWPORT_HEIGHT es la altura que usan los tests.
 */
export const VIEWPORT_HEIGHT = 400;
const VIEWPORT_WIDTH = 1000;

const SIZES: Readonly<Record<string, number>> = {
  offsetHeight: VIEWPORT_HEIGHT,
  clientHeight: VIEWPORT_HEIGHT,
  offsetWidth: VIEWPORT_WIDTH,
  clientWidth: VIEWPORT_WIDTH,
};

const originals = Object.keys(SIZES).map(
  (name) => [name, Object.getOwnPropertyDescriptor(HTMLElement.prototype, name)] as const,
);

beforeEach(() => {
  for (const [name, size] of Object.entries(SIZES)) {
    Object.defineProperty(HTMLElement.prototype, name, { configurable: true, get: () => size });
  }
});

// Vitest corre sin globals, asi que el limpiado de Testing Library se declara
// aqui, y despues se devuelven las medidas originales de jsdom.
afterEach(() => {
  cleanup();
  for (const [name, original] of originals) {
    if (original !== undefined) Object.defineProperty(HTMLElement.prototype, name, original);
  }
});
