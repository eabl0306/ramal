// ESLint de @ramal/bridge: TypeScript estricto, calidad y fronteras. El paquete no
// puede importar nada de npm ni de node:*: los transportes concretos (Tauri,
// postMessage de VS Code) viven en cada app. Lo generado por ts-rs queda fuera
// del lint (DEFAULT_IGNORES); el cliente puede importarlo y nada mas.
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
  { type: 'test-support', pattern: 'src/test' },
  { type: 'client', pattern: 'src' },
];

const fileCategories = [
  { category: 'test', pattern: ['src/**/*.test.ts', 'src/test/**/*'], exclusive: true },
  { category: 'source', pattern: 'src/**/*' },
];

/** Lo que no esta permitido, esta prohibido: ni npm ni node:* en el codigo. */
const policies = [
  {
    from: { element: { type: 'client' }, file: { categories: 'source' } },
    allow: { to: { element: { types: ['client', 'generated'] } } },
  },
  {
    from: { element: { type: 'generated' } },
    allow: { to: { element: { types: ['generated'] } } },
  },
  {
    from: { file: { categories: 'test' } },
    allow: { to: { element: { types: ['client', 'generated', 'test-support'] } } },
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
  ...boundariesConfig({
    rootPath: tsconfigRootDir,
    files: ['src/**/*.ts'],
    elements,
    fileCategories,
    policies,
  }),
  ...testsConfig(),
  ...nodeConfig({ files: ['*.{js,mjs}'] }),
];
