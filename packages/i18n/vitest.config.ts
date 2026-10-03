import { defineConfig } from 'vitest/config';

/**
 * @git-rudder/i18n es nucleo: 80% de cobertura. Lo generado se excluye porque no es
 * logica, solo una tabla que el test de paridad compara entera.
 */
const CORE_THRESHOLD = { lines: 80, statements: 80, branches: 80, functions: 80 };

export default defineConfig({
  test: {
    environment: 'node',
    globals: false,
    include: ['src/**/*.test.ts', 'test/**/*.test.ts'],
    clearMocks: true,
    restoreMocks: true,
    coverage: {
      provider: 'v8',
      reporter: ['text', 'html', 'lcov'],
      reportsDirectory: './coverage',
      include: ['src/**/*.ts'],
      exclude: ['src/**/*.test.ts', 'src/generated/**', 'src/index.ts'],
      thresholds: CORE_THRESHOLD,
    },
  },
});
