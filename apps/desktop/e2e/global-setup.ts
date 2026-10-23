import { build } from 'vite';

import { IPC_MOCK_DIR, IPC_MOCK_FILE } from './support/app';

/**
 * Empaqueta el host simulado (support/ipc-mock.ts, que usa `mockIPC` de
 * @tauri-apps/api/mocks) como un script suelto que cada prueba inyecta con
 * `addInitScript` antes de que cargue la app. Las pruebas no pueden importar
 * modulos dentro de la pagina, y el build de produccion no debe llevar mocks.
 */
export default async function globalSetup(): Promise<void> {
  await build({
    configFile: false,
    logLevel: 'warn',
    build: {
      outDir: IPC_MOCK_DIR,
      emptyOutDir: true,
      minify: false,
      lib: {
        entry: new URL('./support/ipc-mock.ts', import.meta.url).pathname,
        formats: ['iife'],
        name: 'GitRudderIpcMock',
        fileName: () => IPC_MOCK_FILE,
      },
    },
  });
}
