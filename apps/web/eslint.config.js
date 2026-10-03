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

/** Capas de la arquitectura hexagonal, una carpeta por capa (architecture.md 4.6). */
const elements = [
  { type: 'domain', pattern: 'src/domain', partialMatch: false },
  { type: 'application', pattern: 'src/application', partialMatch: false },
  { type: 'infrastructure', pattern: 'src/infrastructure', partialMatch: false },
  { type: 'ui', pattern: 'src/ui', partialMatch: false },
  { type: 'app', pattern: 'src/app', partialMatch: false },
  { type: 'test-support', pattern: 'src/test', partialMatch: false },
];

const LOCAL_TYPES = ['domain', 'application', 'infrastructure', 'ui', 'app', 'test-support'];

/**
 * Los tests son una categoria de fichero, no una capa: pueden montar cualquier
 * adaptador porque ese es justo su trabajo. El codigo de produccion no.
 */
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

/**
 * Lo que cada capa puede importar de npm y del workspace (architecture.md 5.2).
 * Los paquetes @git-rudder/* llegan por node_modules, asi que boundaries los ve como
 * externos y se nombran aqui uno a uno.
 */
function external(type, source) {
  return {
    from: { element: { type }, file: { categories: 'source' } },
    allow: { to: { module: { origin: 'external', source } } },
  };
}

/** Direccion de dependencias. Lo que no esta permitido, esta prohibido. */
const policies = [
  layer('domain', ['domain']),
  layer('application', ['domain', 'application']),
  layer('infrastructure', ['domain', 'application', 'infrastructure']),
  layer('ui', ['domain', 'application', 'ui']),
  layer(
    'app',
    LOCAL_TYPES.filter((type) => type !== 'test-support'),
  ),
  external('domain', ['@git-rudder/domain']),
  external('application', ['@git-rudder/domain']),
  external('infrastructure', ['@git-rudder/domain', '@git-rudder/api-client', 'zod']),
  external('ui', [
    '@git-rudder/domain',
    '@git-rudder/ui',
    '@git-rudder/i18n',
    'react',
    'react-router',
  ]),
  external('app', [
    '@git-rudder/domain',
    '@git-rudder/ui',
    '@git-rudder/ui/**',
    '@git-rudder/i18n',
    'react',
    'react-dom',
    'react-router',
    // Solo lo usa routes.ts, con la subruta /routes; boundaries ve el paquete.
    '@react-router/dev',
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
          origin: 'external',
          source: ['vitest', '@testing-library/**', 'react', 'react-router', '@git-rudder/**'],
        },
      },
    },
  },
];

/**
 * Lo que React Router lee de un modulo de ruta ademas del componente. Sin esta
 * lista, react-refresh pide que el fichero solo exporte componentes.
 */
const ROUTE_EXPORTS = ['meta', 'links', 'loader', 'clientLoader', 'handle', 'shouldRevalidate'];

export default [
  { ignores: [...DEFAULT_IGNORES, 'build/**'] },
  ...baseConfig({ tsconfigRootDir }),
  ...qualityConfig(),
  ...reactConfig({ componentFiles: ['src/**/*.tsx'] }),
  {
    files: ['src/app/root.tsx', 'src/app/routes/**/*.tsx'],
    rules: {
      'react-refresh/only-export-components': ['error', { allowExportNames: ROUTE_EXPORTS }],
    },
  },
  ...boundariesConfig({
    rootPath: tsconfigRootDir,
    files: ['src/**/*.{ts,tsx}'],
    elements,
    fileCategories,
    policies,
    ignore: ['src/**/*.d.ts'],
  }),
  // Los tests montan pantallas con textos de prueba fijos; el producto no.
  ...testsConfig({ allowJsxLiteralText: true }),
  ...nodeConfig({
    files: ['*.{js,mjs}', 'scripts/**/*.mjs', '*.config.ts', 'e2e/**/*.ts'],
  }),
];
