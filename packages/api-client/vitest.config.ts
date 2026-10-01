import { defineConfig } from 'vitest/config';

/**
 * @ramal/api-client no es nucleo: el umbral es el global del monorepo, 60%. Lo
 * generado no se mide, porque son solo tipos que escribe openapi-typescript.
 */
const GLOBAL_THRESHOLD = { lines: 60, statements: 60, branches: 60, functions: 60 };

export default defineConfig({
  test: {
    environment: 'node',
    globals: false,
    include: ['src/**/*.test.ts'],
    clearMocks: true,
    restoreMocks: true,
    coverage: {
      provider: 'v8',
      reporter: ['text', 'html', 'lcov'],
      reportsDirectory: './coverage',
      include: ['src/**/*.ts'],
      exclude: ['src/generated/**', 'src/**/*.test.ts', 'src/index.ts'],
      thresholds: GLOBAL_THRESHOLD,
    },
  },
});
