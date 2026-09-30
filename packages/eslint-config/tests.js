import { LIMITS } from './quality.js';

export const TEST_FILE_PATTERNS = [
  '**/*.test.{ts,tsx,js,mjs}',
  '**/*.spec.{ts,tsx,js,mjs}',
  '**/test/**/*.{ts,tsx,js,mjs}',
  '**/tests/**/*.{ts,tsx,js,mjs}',
  '**/e2e/**/*.{ts,tsx}',
];

/**
 * Los tests se rigen por las mismas reglas salvo donde estorban: un fichero de
 * tests puede ser largo y un bloque describe no es una funcion de produccion.
 * Lo que NO se relaja: complejidad, tests enfocados y marcadores sin ticket.
 *
 * allowJsxLiteralText apaga ramal/no-jsx-literal-text en los tests. No viene
 * encendido por defecto: cada paquete decide si sus tests pueden montar
 * componentes con textos fijos (un fixture) o si tambien deben pasar por i18n.
 *
 * @param {{ files?: string[], allowJsxLiteralText?: boolean }} [options]
 * @returns {import('eslint').Linter.Config[]}
 */
export function testsConfig({ files = TEST_FILE_PATTERNS, allowJsxLiteralText = false } = {}) {
  return [
    {
      files,
      rules: {
        'max-lines': [
          'error',
          { max: LIMITS.maxLinesTest, skipBlankLines: true, skipComments: true },
        ],
        'max-lines-per-function': 'off',
        'max-nested-callbacks': ['error', 5],
        '@typescript-eslint/no-non-null-assertion': 'off',
        '@typescript-eslint/unbound-method': 'off',
        '@typescript-eslint/explicit-module-boundary-types': 'off',
        ...(allowJsxLiteralText ? { 'ramal/no-jsx-literal-text': 'off' } : {}),
      },
    },
  ];
}
