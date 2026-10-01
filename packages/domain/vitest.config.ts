import { defineConfig } from 'vitest/config';

/**
 * El dominio es nucleo: 80% de cobertura en todo, igual que los nucleos de Rust.
 * Corre en node y sin globals porque no hay DOM que probar y los imports de
 * vitest explicitos dejan claro de donde sale cada funcion.
 */
const CORE_THRESHOLD = { lines: 80, statements: 80, branches: 80, functions: 80 };

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
      exclude: ['src/**/*.test.ts', 'src/test/**', 'src/index.ts'],
      thresholds: CORE_THRESHOLD,
    },
  },
});
