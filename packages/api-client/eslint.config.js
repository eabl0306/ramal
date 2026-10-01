// ESLint de @ramal/api-client: TypeScript estricto, calidad y fronteras. El
// codigo solo puede importar openapi-fetch (architecture.md 5.2) y lo generado
// por openapi-typescript, que queda fuera del lint (DEFAULT_IGNORES).
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

/** El orden importa: el primer patron que encaja decide el tipo. */
const elements = [
  { type: 'generated', pattern: 'src/generated' },
  { type: 'client', pattern: 'src' },
];

const fileCategories = [
  { category: 'test', pattern: ['src/**/*.test.ts'], exclusive: true },
  { category: 'source', pattern: 'src/**/*' },
];

/** Lo que no esta permitido, esta prohibido: ni otros paquetes de npm ni node:*. */
const policies = [
  {
    from: { element: { type: 'client' } },
    allow: { to: { element: { types: ['client', 'generated'] } } },
  },
  {
    from: { element: { type: 'client' }, file: { categories: 'source' } },
    allow: { to: { module: { origin: 'external', source: 'openapi-fetch' } } },
  },
  {
    from: { file: { categories: 'test' } },
    allow: { to: { module: { origin: 'external', source: 'vitest' } } },
  },
];

export default [
  { ignores: [...DEFAULT_IGNORES] },
  ...baseConfig({ tsconfigRootDir }),
  ...qualityConfig(),
  ...boundariesConfig({ files: ['src/**/*.ts'], elements, fileCategories, policies }),
  ...testsConfig(),
  ...nodeConfig({ files: ['*.js', 'vitest.config.ts'] }),
];
