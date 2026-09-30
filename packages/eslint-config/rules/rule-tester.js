import { describe, it } from 'node:test';

import { RuleTester } from 'eslint';
import tseslint from 'typescript-eslint';

// RuleTester usa los describe/it globales de mocha o vitest si existen. Con el
// runner de Node no hay globales, asi que se le pasan los de node:test: cada caso
// sale como un test con nombre propio en el informe.
RuleTester.describe = describe;
RuleTester.it = it;

/** RuleTester con el parser de TypeScript y JSX, igual que en los paquetes. */
export function typescriptRuleTester() {
  return new RuleTester({
    languageOptions: {
      parser: tseslint.parser,
      parserOptions: { ecmaFeatures: { jsx: true } },
    },
  });
}

/** RuleTester con el parser por defecto de ESLint, para reglas que no miran tipos. */
export function javascriptRuleTester() {
  return new RuleTester();
}
