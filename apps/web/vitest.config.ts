import react from '@vitejs/plugin-react';
import { defineConfig } from 'vitest/config';

/**
 * Vitest va aparte de vite.config.ts porque el plugin de React Router es para
 * construir la app, no para montar componentes sueltos. Umbrales del monorepo:
 * 80% en el nucleo (domain y application) y 60% en el resto.
 */
const CORE_THRESHOLD = { lines: 80, statements: 80, branches: 80, functions: 80 };

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
      exclude: ['src/test/**', 'src/**/*.test.{ts,tsx}', 'src/**/*.d.ts'],
      thresholds: {
        lines: 60,
        statements: 60,
        branches: 60,
        functions: 60,
        'src/domain/**': CORE_THRESHOLD,
        'src/application/**': CORE_THRESHOLD,
      },
    },
  },
});
