import { fileURLToPath } from 'node:url';

import {
  DEFAULT_IGNORES,
  baseConfig,
  boundariesConfig,
  nodeConfig,
  qualityConfig,
  reactConfig,
  testsConfig,
} from '@ramal/eslint-config';

const tsconfigRootDir = fileURLToPath(new URL('.', import.meta.url));

/**
 * Las piezas de @ramal/repo-view (architecture.md 4.7 y 5.2):
 *
 * - graph: geometria, caminos del SVG y disposicion de la columna del grafo.
 *   Funciones puras sin React; solo leen tipos del protocolo y los tokens.
 * - data: la ventana del motor con TanStack Query sobre el cliente del bridge.
 * - view: componentes React. Son los unicos que pintan.
 *
 * Nada habla con la red: el motor se alcanza solo por el cliente de
 * @ramal/bridge, cuyo transporte pone cada app. Tauri, VS Code y api-client
 * quedan fuera por no estar en ninguna lista blanca.
 */
const elements = [
  { type: 'graph', pattern: 'src/graph', partialMatch: false },
  { type: 'data', pattern: 'src/data', partialMatch: false },
  { type: 'view', pattern: 'src/view', partialMatch: false },
  { type: 'test-support', pattern: 'src/test', partialMatch: false },
  // El resto de src es solo el indice: cada carpeta de arriba es mas concreta.
  { type: 'entry', pattern: 'src', partialMatch: false },
];

const fileCategories = [
  { category: 'test', pattern: ['src/**/*.test.{ts,tsx}', 'src/test/**/*'], exclusive: true },
  { category: 'source', pattern: 'src/**/*' },
];

function internal(type, types) {
  return { from: { element: { type } }, allow: { to: { element: { types } } } };
}

function external(type, source) {
  return {
    from: { element: { type } },
    allow: { to: { module: { origin: 'external', source } } },
  };
}

const policies = [
  internal('graph', ['graph']),
  internal('data', ['data']),
  internal('view', ['view', 'graph', 'data']),
  internal('entry', ['view', 'graph', 'data']),
  external('graph', ['@ramal/bridge', '@ramal/ui']),
  external('data', ['@ramal/bridge', '@ramal/i18n', '@tanstack/react-query', 'react']),
  external('view', [
    'react',
    '@ramal/bridge',
    '@ramal/i18n',
    '@ramal/ui',
    '@tanstack/react-query',
    '@tanstack/react-virtual',
  ]),
  external('entry', ['@ramal/bridge']),
  {
    from: { file: { categories: 'test' } },
    allow: { to: { element: { types: ['graph', 'data', 'view', 'test-support'] } } },
  },
  {
    from: { file: { categories: 'test' } },
    allow: {
      to: {
        module: {
          origin: ['external', 'core'],
          source: [
            'vitest',
            '@testing-library/**',
            'react',
            '@ramal/bridge',
            '@ramal/i18n',
            '@ramal/ui',
            '@ramal/ui/icons/lucide',
            '@ramal/ui/icons/codicons',
            '@tanstack/react-query',
            // Solo para leer las hojas de src/view: jsdom no aplica los CSS Modules
            // y el diseno (alto contraste, tintes) solo se comprueba en la regla.
            'node:fs',
            'node:path',
            'node:url',
          ],
        },
      },
    },
  },
];

/** La vista nunca abre conexiones: toda salida pasa por el cliente del bridge. */
const NETWORK_GLOBALS = ['fetch', 'XMLHttpRequest', 'WebSocket', 'EventSource'].map((name) => ({
  name,
  message: '@ramal/repo-view no habla con la red: usa el cliente de @ramal/bridge.',
}));

export default [
  { ignores: [...DEFAULT_IGNORES] },
  ...baseConfig({ tsconfigRootDir }),
  ...qualityConfig(),
  ...reactConfig({ componentFiles: ['src/**/*.tsx'] }),
  ...boundariesConfig({
    files: ['src/**/*.{ts,tsx}'],
    elements,
    fileCategories,
    policies,
    ignore: ['src/**/*.d.ts'],
  }),
  {
    files: ['src/**/*.{ts,tsx}'],
    rules: { 'no-restricted-globals': ['error', ...NETWORK_GLOBALS] },
  },
  // Los tests montan componentes con datos de prueba fijos; el producto no.
  ...testsConfig({ allowJsxLiteralText: true }),
  ...nodeConfig({ files: ['*.js', 'vitest.config.ts'] }),
];
