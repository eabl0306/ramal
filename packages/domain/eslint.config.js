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
 * El paquete entero es una sola capa: dominio puro. Cada carpeta de src es un
 * modulo de esa capa y puede usar a los demas, pero ninguno puede importar
 * nada de fuera, ni de npm ni de node:*.
 */
const elements = [{ type: 'domain', pattern: 'src', partialMatch: false }];

/** Los tests son una categoria aparte: pueden importar vitest. */
const fileCategories = [
  { category: 'test', pattern: ['src/**/*.test.ts', 'src/test/**/*'], exclusive: true },
  { category: 'source', pattern: 'src/**/*' },
];

const policies = [
  {
    from: { element: { type: 'domain' } },
    allow: { to: { element: { types: ['domain'] } } },
  },
  {
    from: { file: { categories: 'test' } },
    allow: { to: { module: { origin: 'external', source: 'vitest' } } },
  },
  {
    from: { element: { type: 'domain' }, file: { categories: 'source' } },
    disallow: { to: { module: { origin: ['external', 'core'] } } },
    message: '@ramal/domain es puro: sin paquetes de npm ni node:*.',
  },
];

export default [
  { ignores: [...DEFAULT_IGNORES] },
  ...baseConfig({ tsconfigRootDir }),
  ...qualityConfig(),
  ...boundariesConfig({
    files: ['src/**/*.ts'],
    elements,
    fileCategories,
    policies,
  }),
  ...testsConfig(),
  ...nodeConfig({ files: ['*.js', 'vitest.config.ts'] }),
];
