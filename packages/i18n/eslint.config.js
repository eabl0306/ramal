import { fileURLToPath } from 'node:url';

import {
  DEFAULT_IGNORES,
  baseConfig,
  boundariesConfig,
  nodeConfig,
  qualityConfig,
  testsConfig,
} from '@ramal/eslint-config';

const tsconfigRootDir = fileURLToPath(new URL('.', import.meta.url));

/**
 * Una sola capa, biblioteca pura: el codigo de src no importa nada de fuera, ni
 * React (su enlace vive en @ramal/ui) ni node:*. Los tests si pueden usar
 * vitest y leer del disco.
 */
const elements = [
  { type: 'i18n', pattern: 'src', partialMatch: false },
  { type: 'i18n-test', pattern: 'test', partialMatch: false },
];

const fileCategories = [
  { category: 'test', pattern: ['src/**/*.test.ts', 'test/**/*'], exclusive: true },
  { category: 'source', pattern: 'src/**/*' },
];

const policies = [
  {
    from: { element: { type: 'i18n' } },
    allow: { to: { element: { types: ['i18n'] } } },
  },
  {
    from: { file: { categories: 'test' } },
    allow: { to: { element: { types: ['i18n'] } } },
  },
  {
    from: { file: { categories: 'test' } },
    allow: { to: { module: { origin: ['external', 'core'], source: ['vitest', 'node:*'] } } },
  },
  {
    from: { element: { type: 'i18n' }, file: { categories: 'source' } },
    disallow: { to: { module: { origin: ['external', 'core'] } } },
    message: '@ramal/i18n es puro: sin paquetes de npm ni node:*.',
  },
];

export default [
  { ignores: [...DEFAULT_IGNORES] },
  ...baseConfig({ tsconfigRootDir }),
  ...qualityConfig(),
  ...boundariesConfig({
    rootPath: tsconfigRootDir,
    files: ['src/**/*.ts', 'test/**/*.ts'],
    elements,
    fileCategories,
    policies,
    include: ['src/**/*', 'test/**/*'],
    extensions: ['.ts', '.json'],
  }),
  ...testsConfig(),
  ...nodeConfig({ files: ['*.js', 'scripts/**/*.mjs', 'vitest.config.ts'] }),
];
