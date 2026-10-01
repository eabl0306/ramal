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
 * Las piezas de @ramal/ui y lo que puede importar cada una (architecture.md
 * 5.2): lucide-react solo en icons/lucide, @vscode/codicons solo en
 * icons/codicons y @fontsource solo en themes, para que ningun host cargue el
 * juego de iconos o las fuentes de otro. @ramal/i18n solo entra por el enlace
 * de React (src/i18n).
 */
const elements = [
  { type: 'icons-lucide', pattern: 'src/icons/lucide', partialMatch: false },
  { type: 'icons-codicons', pattern: 'src/icons/codicons', partialMatch: false },
  { type: 'icons', pattern: 'src/icons', partialMatch: false },
  { type: 'themes', pattern: 'src/themes', partialMatch: false },
  { type: 'tokens', pattern: 'src/tokens', partialMatch: false },
  { type: 'i18n-binding', pattern: 'src/i18n', partialMatch: false },
  { type: 'components', pattern: 'src/components', partialMatch: false },
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
  internal('tokens', ['tokens']),
  internal('themes', ['themes']),
  internal('i18n-binding', ['i18n-binding']),
  internal('icons', ['icons']),
  internal('icons-lucide', ['icons']),
  internal('icons-codicons', ['icons']),
  internal('components', ['components', 'icons', 'i18n-binding', 'tokens']),
  internal('entry', ['components', 'icons', 'i18n-binding', 'tokens']),
  external('themes', ['@fontsource/**']),
  external('i18n-binding', ['react', '@ramal/i18n', 'react-aria-components']),
  external('icons', ['react']),
  external('icons-lucide', ['react', 'lucide-react']),
  external('icons-codicons', ['@vscode/codicons/**']),
  external('components', ['react', 'react-dom', 'react-aria-components']),
  {
    from: { file: { categories: 'test' } },
    allow: {
      to: {
        element: {
          types: [
            'tokens',
            'themes',
            'i18n-binding',
            'icons',
            'icons-lucide',
            'icons-codicons',
            'components',
            'test-support',
          ],
        },
      },
    },
  },
  {
    from: { file: { categories: 'test' } },
    allow: {
      to: {
        module: {
          origin: ['external', 'core'],
          source: ['vitest', '@testing-library/**', 'react', '@ramal/i18n', 'node:*'],
        },
      },
    },
  },
];

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
  // Los tests montan componentes con textos de prueba fijos; el producto no.
  ...testsConfig({ allowJsxLiteralText: true }),
  ...nodeConfig({ files: ['*.js', 'vitest.config.ts'] }),
];
