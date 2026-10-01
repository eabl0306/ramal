import react from '@vitejs/plugin-react';
import { defineConfig } from 'vitest/config';

/**
 * Dos proyectos con entornos distintos: el extension host y los mensajes
 * compartidos en Node, los webviews en jsdom. La cobertura es una sola.
 *
 * Umbrales (architecture.md 5.3): 80% en la aplicacion del extension host, que
 * es su nucleo, y 60% global. Fuera de la cobertura unitaria queda lo que solo
 * se puede ejecutar dentro de VS Code o de un navegador: los adaptadores de la
 * API de VS Code y el composition root (los cubren los tests de integracion bajo
 * xvfb) y las entradas de los webviews (las cubre Playwright).
 */
const CORE_THRESHOLD = { lines: 80, statements: 80, branches: 80, functions: 80 };
const GLOBAL_THRESHOLD = { lines: 60, statements: 60, branches: 60, functions: 60 };

export default defineConfig({
  test: {
    clearMocks: true,
    restoreMocks: true,
    projects: [
      {
        test: {
          name: 'extension',
          environment: 'node',
          include: ['src/extension/**/*.test.ts', 'src/shared/**/*.test.ts'],
        },
      },
      {
        plugins: [react()],
        test: {
          name: 'webview',
          environment: 'jsdom',
          setupFiles: ['./src/test/setup.ts'],
          include: ['src/webview/**/*.test.{ts,tsx}'],
        },
      },
    ],
    coverage: {
      provider: 'v8',
      reporter: ['text', 'html', 'lcov'],
      reportsDirectory: './coverage',
      include: ['src/**/*.{ts,tsx}'],
      exclude: [
        'src/**/*.test.{ts,tsx}',
        'src/test/**',
        'src/**/*.d.ts',
        'src/extension/infrastructure/vscode/**',
        'src/extension/app/**',
        'src/webview/app/**',
      ],
      thresholds: {
        ...GLOBAL_THRESHOLD,
        'src/extension/application/**': CORE_THRESHOLD,
      },
    },
  },
});
