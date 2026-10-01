// Vitest de @ramal/bridge. Es JavaScript a proposito: el tsconfig del paquete es
// una libreria pura (sin tipos de Node) que solo cubre src/, y este fichero corre
// en Node.
import { defineConfig } from 'vitest/config';

/** Minimo de cobertura del paquete en las cuatro metricas. */
const THRESHOLD = 80;

export default defineConfig({
  test: {
    environment: 'node',
    include: ['src/**/*.test.ts'],
    clearMocks: true,
    restoreMocks: true,
    coverage: {
      provider: 'v8',
      reporter: ['text', 'lcov'],
      reportsDirectory: './coverage',
      include: ['src/**/*.ts'],
      exclude: ['src/generated/**', 'src/test/**', 'src/**/*.test.ts', 'src/index.ts'],
      thresholds: {
        lines: THRESHOLD,
        statements: THRESHOLD,
        branches: THRESHOLD,
        functions: THRESHOLD,
      },
    },
  },
});
