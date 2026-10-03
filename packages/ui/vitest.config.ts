import react from '@vitejs/plugin-react';
import { defineConfig } from 'vitest/config';

/**
 * Componentes con Testing Library sobre jsdom. @git-rudder/ui no es nucleo: el umbral
 * es el global del monorepo, 60%. Los ficheros de solo tipos y el indice no
 * tienen logica que medir.
 */
const GLOBAL_THRESHOLD = { lines: 60, statements: 60, branches: 60, functions: 60 };

export default defineConfig({
  plugins: [react()],
  test: {
    environment: 'jsdom',
    globals: false,
    setupFiles: ['./src/test/setup.ts'],
    include: ['src/**/*.test.{ts,tsx}'],
    clearMocks: true,
    restoreMocks: true,
    coverage: {
      provider: 'v8',
      reporter: ['text', 'html', 'lcov'],
      reportsDirectory: './coverage',
      include: ['src/**/*.{ts,tsx}'],
      exclude: ['src/**/*.test.{ts,tsx}', 'src/test/**', 'src/**/*.d.ts', 'src/index.ts'],
      thresholds: GLOBAL_THRESHOLD,
    },
  },
});
