import '@testing-library/jest-dom/vitest';

import { cleanup } from '@testing-library/react';
import { afterEach, beforeEach } from 'vitest';

/**
 * jsdom no maqueta: todo mide 0 px y el virtualizador no pintaria ninguna fila.
 * Se da un tamano fijo al contenedor con scroll, que es lo unico que lee
 * @tanstack/virtual-core (offsetWidth y offsetHeight), para que los tests vean
 * las filas de la primera pantalla. VIEWPORT_HEIGHT es la altura que usan.
 */
export const VIEWPORT_HEIGHT = 400;
const VIEWPORT_WIDTH = 1000;

const originals = {
  height: Object.getOwnPropertyDescriptor(HTMLElement.prototype, 'offsetHeight'),
  width: Object.getOwnPropertyDescriptor(HTMLElement.prototype, 'offsetWidth'),
};

beforeEach(() => {
  Object.defineProperty(HTMLElement.prototype, 'offsetHeight', {
    configurable: true,
    get: () => VIEWPORT_HEIGHT,
  });
  Object.defineProperty(HTMLElement.prototype, 'offsetWidth', {
    configurable: true,
    get: () => VIEWPORT_WIDTH,
  });
});

// Vitest corre sin globals, asi que el limpiado de Testing Library se declara
// aqui, y despues se devuelven las medidas originales de jsdom.
afterEach(() => {
  cleanup();
  if (originals.height !== undefined) {
    Object.defineProperty(HTMLElement.prototype, 'offsetHeight', originals.height);
  }
  if (originals.width !== undefined) {
    Object.defineProperty(HTMLElement.prototype, 'offsetWidth', originals.width);
  }
});
