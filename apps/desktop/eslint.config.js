import { fileURLToPath } from 'node:url';

import {
  DEFAULT_IGNORES,
  baseConfig,
  boundariesConfig,
  nodeConfig,
  qualityConfig,
  reactConfig,
  testsConfig,
} from '@git-rudder/eslint-config';

const tsconfigRootDir = fileURLToPath(new URL('.', import.meta.url));

/**
 * Las capas del frontend de escritorio (architecture.md 4.3 y 5.2). No hay
 * `domain` propio: el dominio del cliente esta en el motor de Rust.
 *
 * - application: puertos del host (RepositoryPicker, AppearanceStore) y la
 *   orquestacion de pantalla (pestanas). Sin React ni Tauri.
 * - infrastructure: adaptadores de Tauri (TauriTransport, el selector) y del
 *   almacenamiento. Sin React.
 * - ui: pantallas propias; reciben las capacidades por contexto y nunca importan
 *   un adaptador ni Tauri.
 * - app: composition root, proveedores y entrada.
 */
const elements = [
  { type: 'application', pattern: 'src/application', partialMatch: false },
  { type: 'infrastructure', pattern: 'src/infrastructure', partialMatch: false },
  { type: 'ui', pattern: 'src/ui', partialMatch: false },
  { type: 'app', pattern: 'src/app', partialMatch: false },
  { type: 'test-support', pattern: 'src/test', partialMatch: false },
];

const LOCAL_TYPES = ['application', 'infrastructure', 'ui', 'app', 'test-support'];

/** Los tests son una categoria de fichero, no una capa: pueden montar cualquier adaptador. */
const fileCategories = [
  { category: 'test', pattern: ['src/**/*.test.{ts,tsx}', 'src/test/**/*'], exclusive: true },
  { category: 'source', pattern: 'src/**/*' },
];

const TAURI_MODULES = ['@tauri-apps/api', '@tauri-apps/api/*', '@tauri-apps/plugin-*'];
const SHARED_UI = [
  '@git-rudder/ui',
  '@git-rudder/repo-view',
  '@git-rudder/i18n',
  '@git-rudder/domain',
];

function internal(type, types) {
  return {
    from: { element: { type }, file: { categories: 'source' } },
    allow: { to: { element: { types } } },
  };
}

function external(type, source) {
  return {
    from: { element: { type }, file: { categories: 'source' } },
    allow: { to: { module: { origin: 'external', source } } },
  };
}

/** Solo los tipos de un paquete: `import type`, nunca su codigo. */
function typesOnly(type, source) {
  return {
    from: { element: { type }, file: { categories: 'source' } },
    allow: { to: { module: { origin: 'external', source } }, dependency: { kind: 'type' } },
  };
}

/** Direccion de dependencias. Lo que no esta permitido, esta prohibido. */
const policies = [
  internal('application', ['application']),
  internal('infrastructure', ['application', 'infrastructure']),
  internal('ui', ['application', 'ui']),
  internal('app', ['application', 'infrastructure', 'ui', 'app']),
  external('application', ['@git-rudder/domain']),
  typesOnly('application', ['@git-rudder/bridge']),
  external('infrastructure', ['@git-rudder/bridge', '@git-rudder/domain', ...TAURI_MODULES]),
  external('ui', ['react', ...SHARED_UI]),
  typesOnly('ui', ['@git-rudder/bridge']),
  external('app', [
    'react',
    'react-dom',
    'react-dom/client',
    '@tanstack/react-query',
    '@git-rudder/bridge',
    ...SHARED_UI,
    '@git-rudder/ui/icons/lucide',
    '@git-rudder/ui/themes/git-rudder.css',
    '@git-rudder/ui/themes/fonts',
  ]),
  {
    from: { file: { categories: 'test' } },
    allow: { to: { element: { types: LOCAL_TYPES } } },
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
            '@tanstack/react-query',
            '@git-rudder/bridge',
            '@git-rudder/ui/icons/lucide',
            ...SHARED_UI,
          ],
        },
      },
    },
  },
];

/** El webview no habla con la red: toda salida pasa por el comando `rpc` (CSP de 6.8). */
const NETWORK_GLOBALS = ['fetch', 'XMLHttpRequest', 'WebSocket', 'EventSource'].map((name) => ({
  name,
  message: 'El webview de escritorio no habla con la red: usa el motor por el comando rpc.',
}));

const NODE_FILES = [
  '*.js',
  'vite.config.ts',
  'playwright.config.ts',
  'wdio.conf.ts',
  'e2e/**/*.ts',
  'wdio/**/*.ts',
];

export default [
  { ignores: [...DEFAULT_IGNORES, 'src-tauri/**'] },
  ...baseConfig({ tsconfigRootDir }),
  ...qualityConfig(),
  ...reactConfig({ files: ['src/**/*.{ts,tsx}'], componentFiles: ['src/**/*.tsx'] }),
  ...boundariesConfig({
    rootPath: tsconfigRootDir,
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
  ...nodeConfig({ files: NODE_FILES }),
];
