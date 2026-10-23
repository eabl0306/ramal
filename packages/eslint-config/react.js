import eslintReact from '@eslint-react/eslint-plugin';
import jsxA11y from 'eslint-plugin-jsx-a11y';
import reactHooks from 'eslint-plugin-react-hooks';
import reactRefresh from 'eslint-plugin-react-refresh';
import globals from 'globals';

import { gitRudderPlugin } from './rules/plugin.js';

/**
 * @eslint-react trae su propia copia de las reglas de hooks y del compilador.
 * Las oficiales son las de eslint-plugin-react-hooks, que mantiene el equipo de
 * React, asi que estas se apagan para no ver cada fallo dos veces.
 */
const DUPLICATED_HOOK_RULES = [
  'error-boundaries',
  'exhaustive-deps',
  'purity',
  'rules-of-hooks',
  'set-state-in-effect',
  'set-state-in-render',
  'static-components',
  'unsupported-syntax',
  'use-memo',
];

const duplicatedHookRulesOff = Object.fromEntries(
  DUPLICATED_HOOK_RULES.map((rule) => [`@eslint-react/${rule}`, 'off']),
);

/**
 * Capa de React: reglas de hooks, del compilador, de componentes y de
 * accesibilidad, mas el entorno de navegador y la regla de textos visibles. El
 * estilo del JSX no se toca: de eso vive Prettier.
 *
 * eslint-plugin-react no entra: su ultima version no soporta ESLint 10. Su hueco lo
 * cubre @eslint-react, que si lo soporta y necesita la informacion de tipos de
 * baseConfig, asi que esta capa va siempre detras de ella.
 *
 * Sobre accesibilidad: eslint-plugin-jsx-a11y 6.10.2 todavia no declara ESLint 10
 * en sus peers, pero sus reglas funcionan con el. La excepcion esta escrita en
 * pnpm-workspace.yaml (peerDependencyRules) en lugar de dejarse como aviso suelto.
 *
 * @param {{ files?: string[], componentFiles?: string[], refresh?: boolean }} [options]
 *   componentFiles son los ficheros con JSX, donde se vigilan los textos visibles
 *   y los exports compatibles con el recargado en caliente. refresh se apaga en
 *   codigo que no pasa por Vite, como el host de la extension.
 * @returns {import('eslint').Linter.Config[]}
 */
export function reactConfig({
  files = ['**/*.{ts,tsx}'],
  componentFiles = ['**/*.tsx'],
  refresh = true,
} = {}) {
  return [
    {
      files,
      languageOptions: {
        globals: { ...globals.browser },
        parserOptions: { ecmaFeatures: { jsx: true } },
      },
    },
    { ...eslintReact.configs['recommended-type-checked'], files },
    { files, rules: duplicatedHookRulesOff },
    { ...reactHooks.configs.flat['recommended-latest'], files },
    { ...jsxA11y.flatConfigs.recommended, files },
    ...(refresh ? [{ ...reactRefresh.configs.vite, files: componentFiles }] : []),
    {
      files,
      rules: {
        'react-hooks/exhaustive-deps': 'error',
        'react-hooks/rules-of-hooks': 'error',
      },
    },
    {
      files: componentFiles,
      plugins: { 'git-rudder': gitRudderPlugin },
      rules: { 'git-rudder/no-jsx-literal-text': 'error' },
    },
  ];
}
