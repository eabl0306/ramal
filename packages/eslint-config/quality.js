import sonarjs from 'eslint-plugin-sonarjs';

import { gitRudderPlugin } from './rules/plugin.js';

/** Limites duros del monorepo. Cambiarlos es una decision de equipo, no un ajuste local. */
export const LIMITS = {
  maxLines: 300,
  maxLinesTest: 400,
  maxLinesPerFunction: 50,
  complexity: 10,
  cognitiveComplexity: 15,
  maxDepth: 4,
  maxParams: 5,
  maxNestedCallbacks: 3,
};

const focusedTestSelectors = [
  {
    selector:
      "CallExpression[callee.type='MemberExpression'][callee.property.name='only']" +
      '[callee.object.name=/^(describe|it|test|suite|bench)$/]',
    message: 'Test enfocado (.only). Quitalo antes de commitear.',
  },
  {
    selector:
      "CallExpression[callee.type='MemberExpression'][callee.property.name='only']" +
      "[callee.object.type='MemberExpression'][callee.object.property.name='describe']",
    message: 'Test enfocado (.only). Quitalo antes de commitear.',
  },
  {
    selector: 'CallExpression[callee.name=/^(fdescribe|fit)$/]',
    message: 'Test enfocado. Quitalo antes de commitear.',
  },
];

/**
 * Capa de calidad: tamano de fichero, complejidad, anidamiento y codigo muerto.
 * Se aplica a todo el JavaScript y TypeScript del repo.
 *
 * @param {{ files?: string[] }} [options]
 * @returns {import('eslint').Linter.Config[]}
 */
export function qualityConfig({ files = ['**/*.{js,mjs,cjs,ts,tsx,mts,cts}'] } = {}) {
  return [
    {
      files,
      plugins: { sonarjs, 'git-rudder': gitRudderPlugin },
      rules: {
        'max-lines': ['error', { max: LIMITS.maxLines, skipBlankLines: true, skipComments: true }],
        'max-lines-per-function': [
          'error',
          { max: LIMITS.maxLinesPerFunction, skipBlankLines: true, skipComments: true },
        ],
        complexity: ['error', LIMITS.complexity],
        'max-depth': ['error', LIMITS.maxDepth],
        'max-params': ['error', LIMITS.maxParams],
        'max-nested-callbacks': ['error', LIMITS.maxNestedCallbacks],
        'sonarjs/cognitive-complexity': ['error', LIMITS.cognitiveComplexity],
        'sonarjs/no-identical-functions': 'error',
        'sonarjs/no-collapsible-if': 'error',
        'sonarjs/no-redundant-jump': 'error',
        'sonarjs/prefer-immediate-return': 'error',

        'no-console': 'error',
        'no-debugger': 'error',
        'no-alert': 'error',
        'no-unreachable': 'error',
        'no-unused-private-class-members': 'error',
        'no-param-reassign': ['error', { props: true }],
        'no-restricted-syntax': ['error', ...focusedTestSelectors],
        'git-rudder/ticketed-todo': 'error',
        eqeqeq: ['error', 'always', { null: 'ignore' }],
        curly: ['error', 'multi-line'],
        'prefer-const': 'error',
        'no-var': 'error',
      },
    },
  ];
}
