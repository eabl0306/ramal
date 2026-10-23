import js from '@eslint/js';
import prettier from 'eslint-config-prettier/flat';
import { createTypeScriptImportResolver } from 'eslint-import-resolver-typescript';
import importX from 'eslint-plugin-import-x';
import tseslint from 'typescript-eslint';

import { gitRudderPlugin } from './rules/plugin.js';

/**
 * Directorios que nunca contienen codigo que revisar. Los patrones empiezan por
 * `**` para valer igual desde el eslint.config.js de cualquier paquete.
 */
export const DEFAULT_IGNORES = [
  '**/dist/**',
  '**/build/**',
  '**/out/**',
  '**/coverage/**',
  '**/node_modules/**',
  '**/target/**',
  '**/playwright-report/**',
  '**/test-results/**',
  '**/.vite/**',
  '**/.vscode-test/**',
  '**/.react-router/**',
  // Worktrees de los agentes: copias completas del repo con su propio codigo.
  '**/.claude/worktrees/**',
  // Codigo generado: los tipos del protocolo (ts-rs) y el cliente de la API
  // (OpenAPI). Se corrigen en su generador, no a mano.
  '**/generated/**',
  '**/src-tauri/gen/**',
  '**/*.d.ts',
];

const importRules = {
  'import-x/no-cycle': ['error', { maxDepth: Infinity, ignoreExternal: true }],
  'import-x/no-self-import': 'error',
  'import-x/no-duplicates': 'error',
  'import-x/no-useless-path-segments': ['error', { noUselessIndex: true }],
  'import-x/first': 'error',
  'import-x/newline-after-import': 'error',
  'import-x/order': [
    'error',
    {
      groups: ['builtin', 'external', 'internal', 'parent', 'sibling', 'index'],
      'newlines-between': 'always',
      alphabetize: { order: 'asc', caseInsensitive: true },
    },
  ],
};

const typescriptRules = {
  '@typescript-eslint/no-unused-vars': [
    'error',
    {
      args: 'all',
      argsIgnorePattern: '^_',
      varsIgnorePattern: '^_',
      caughtErrorsIgnorePattern: '^_',
      ignoreRestSiblings: true,
    },
  ],
  '@typescript-eslint/consistent-type-imports': [
    'error',
    { prefer: 'type-imports', fixStyle: 'inline-type-imports' },
  ],
  '@typescript-eslint/no-import-type-side-effects': 'error',
  '@typescript-eslint/explicit-module-boundary-types': 'error',
  '@typescript-eslint/no-floating-promises': 'error',
  '@typescript-eslint/no-misused-promises': 'error',
  '@typescript-eslint/switch-exhaustiveness-check': 'error',
  '@typescript-eslint/no-unnecessary-condition': 'error',
  '@typescript-eslint/strict-boolean-expressions': [
    'error',
    { allowNullableBoolean: true, allowNullableString: true },
  ],
};

/**
 * Capa base: JavaScript recomendado, TypeScript type-aware en modo estricto,
 * orden de imports, colores solo desde los tokens y desactivacion de todo lo que
 * pelea con Prettier.
 *
 * @param {{ tsconfigRootDir: string, files?: string[] }} options
 * @returns {import('eslint').Linter.Config[]}
 */
export function baseConfig({ tsconfigRootDir, files = ['**/*.{ts,tsx,mts,cts}'] }) {
  return tseslint.config(
    { files, extends: [js.configs.recommended] },
    { files, extends: tseslint.configs.strictTypeChecked },
    { files, extends: tseslint.configs.stylisticTypeChecked },
    {
      files,
      extends: [importX.flatConfigs.recommended, importX.flatConfigs.typescript],
      languageOptions: {
        parserOptions: { projectService: true, tsconfigRootDir },
      },
      settings: {
        'import-x/resolver-next': [createTypeScriptImportResolver({ alwaysTryTypes: true })],
      },
      rules: { ...importRules, ...typescriptRules },
    },
    {
      files,
      plugins: { 'git-rudder': gitRudderPlugin },
      // La excepcion de packages/ui/src/themes la aplica la propia regla mirando la
      // ruta del fichero, asi funciona igual desde cualquier eslint.config.js.
      rules: { 'git-rudder/no-raw-colors': 'error' },
    },
    { files, extends: [prettier] },
  );
}
