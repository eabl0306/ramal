// ESLint de la extension de VS Code: TypeScript estricto, calidad y las capas de
// architecture.md 5.2 con eslint-plugin-boundaries. Dos mitades con entornos
// distintos: el extension host (Node, sin DOM) y los webviews (navegador, React).
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

/** Una carpeta por capa; el primer patron que encaja decide. */
const elements = [
  { type: 'shared', pattern: 'src/shared', partialMatch: false },
  { type: 'ext-application', pattern: 'src/extension/application', partialMatch: false },
  { type: 'ext-infrastructure', pattern: 'src/extension/infrastructure', partialMatch: false },
  { type: 'ext-app', pattern: 'src/extension/app', partialMatch: false },
  { type: 'web-infrastructure', pattern: 'src/webview/infrastructure', partialMatch: false },
  { type: 'web-ui', pattern: 'src/webview/ui', partialMatch: false },
  { type: 'web-app', pattern: 'src/webview/app', partialMatch: false },
  { type: 'test-support', pattern: 'src/test', partialMatch: false },
];

const LOCAL_TYPES = elements.map((element) => element.type);

/** Los tests son una categoria de fichero, no una capa: pueden montar cualquier pieza. */
const fileCategories = [
  { category: 'test', pattern: ['src/**/*.test.{ts,tsx}', 'src/test/**/*'], exclusive: true },
  { category: 'source', pattern: 'src/**/*' },
];

function layer(type, types) {
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

/**
 * Solo los tipos de un paquete: `import type`, nunca su codigo. Con `external`
 * la UI de un webview podia importar `createBridgeClient` y montar clientes por
 * su cuenta sin que el lint dijera nada; escritorio ya lo cerraba asi.
 */
function typesOnly(type, source) {
  return {
    from: { element: { type }, file: { categories: 'source' } },
    allow: { to: { module: { origin: 'external', source } }, dependency: { kind: 'type' } },
  };
}

function core(type) {
  return {
    from: { element: { type }, file: { categories: 'source' } },
    allow: { to: { module: { origin: 'core' } } },
  };
}

/**
 * La tabla de 5.2. Lo que no esta permitido, esta prohibido: `vscode` y `node:*`
 * solo en la infraestructura del host (y `vscode` en su composition root), zod
 * solo donde se valida, y nada de React en el extension host.
 */
const policies = [
  layer('shared', ['shared']),
  typesOnly('shared', ['@ramal/bridge']),
  layer('ext-application', ['shared', 'ext-application']),
  external('ext-application', ['@ramal/i18n', '@ramal/bridge']),
  layer('ext-infrastructure', ['shared', 'ext-application', 'ext-infrastructure']),
  external('ext-infrastructure', ['@ramal/bridge', 'vscode', 'zod']),
  core('ext-infrastructure'),
  layer('ext-app', ['shared', 'ext-application', 'ext-infrastructure', 'ext-app']),
  external('ext-app', ['vscode']),
  layer('web-infrastructure', ['shared', 'web-infrastructure']),
  external('web-infrastructure', ['@ramal/bridge', '@ramal/domain']),
  layer('web-ui', ['web-ui']),
  external('web-ui', [
    'react',
    '@ramal/ui',
    '@ramal/ui/**',
    '@ramal/repo-view',
    '@ramal/i18n',
    '@ramal/domain',
  ]),
  typesOnly('web-ui', ['@ramal/bridge']),
  layer('web-app', ['shared', 'web-infrastructure', 'web-ui', 'web-app']),
  external('web-app', ['react', 'react-dom', 'react-dom/client', '@tanstack/react-query']),
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
            '@ramal/bridge',
            '@ramal/i18n',
            'node:*',
          ],
        },
      },
    },
  },
];

const HOST_FILES = ['src/extension/**/*.ts'];
const WEBVIEW_FILES = ['src/webview/**/*.{ts,tsx}'];

export default [
  { ignores: [...DEFAULT_IGNORES] },
  ...baseConfig({ tsconfigRootDir }),
  ...qualityConfig(),
  ...nodeConfig({ files: [...HOST_FILES, 'src/shared/**/*.ts'] }),
  ...reactConfig({ files: WEBVIEW_FILES, componentFiles: ['src/webview/**/*.tsx'] }),
  ...boundariesConfig({
    rootPath: tsconfigRootDir,
    files: ['src/**/*.{ts,tsx}'],
    elements,
    fileCategories,
    policies,
    ignore: ['src/**/*.d.ts'],
  }),
  {
    // shared solo puede ver los tipos de @ramal/bridge: ningun valor de un
    // paquete entra en los dos lados por la puerta de atras.
    files: ['src/shared/**/*.ts'],
    rules: {
      '@typescript-eslint/no-restricted-imports': [
        'error',
        { paths: [{ name: '@ramal/bridge', allowTypeImports: true }] },
      ],
    },
  },
  ...testsConfig({ allowJsxLiteralText: true }),
  ...nodeConfig({
    files: ['*.{js,mjs,ts}', '.vscode-test.mjs', 'scripts/**/*.mjs', 'e2e/**/*.ts'],
  }),
  {
    files: ['test/integration/**/*.ts'],
    languageOptions: {
      globals: { suite: 'readonly', test: 'readonly', suiteSetup: 'readonly', setup: 'readonly' },
    },
  },
];
