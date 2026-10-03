// ESLint de la raiz: cubre la configuracion compartida (packages/eslint-config)
// y los scripts del monorepo (tools/), que son JavaScript puro. Cada app y cada
// paquete de TypeScript trae su propio eslint.config.js con las reglas
// type-aware, de React y de arquitectura; ESLint 10 busca la configuracion desde
// el directorio de cada fichero, asi que aqui solo se aplican la capa
// recomendada, la de Node, la de calidad y la de tests.
import js from '@eslint/js';
import { DEFAULT_IGNORES, nodeConfig, qualityConfig, testsConfig } from '@git-rudder/eslint-config';

/** Lo que linta esta configuracion. apps/ queda fuera: cada app trae la suya. */
const FILES = [
  '*.{js,mjs,cjs}',
  'packages/**/*.{js,mjs,cjs}',
  'tools/**/*.{js,mjs,cjs}',
  '.github/**/*.{js,mjs,cjs}',
];

export default [
  {
    ignores: [
      ...DEFAULT_IGNORES,
      // docs/ no se linta: el prototipo de diseno es material de referencia tal cual llego.
      'docs/**',
      // Las apps y los paquetes de TypeScript traen su propio eslint.config.js y los
      // lintan sus recetas (just packages-check, just web-check...). Sin estas lineas
      // `eslint .` en la raiz tambien entraria en ellos con su configuracion, y el lint
      // de la raiz dependeria de que cada paquete tenga su typecheck en orden.
      'apps/**',
      'packages/*',
      '!packages/eslint-config',
    ],
  },
  { files: FILES, ...js.configs.recommended },
  ...nodeConfig({ files: FILES }),
  {
    files: FILES,
    rules: {
      'no-unused-vars': [
        'error',
        {
          args: 'all',
          argsIgnorePattern: '^_',
          varsIgnorePattern: '^_',
          caughtErrorsIgnorePattern: '^_',
          ignoreRestSiblings: true,
        },
      ],
    },
  },
  ...qualityConfig({ files: FILES }),
  ...testsConfig({ files: ['tools/**/*.test.{js,mjs}', 'packages/eslint-config/**/*.test.js'] }),
];
