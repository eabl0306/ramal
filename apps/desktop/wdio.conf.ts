import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { browser } from '@wdio/globals';

import { type FixtureRepo, createFixtureRepo } from './wdio/support/fixture-repo';

/**
 * E2E nativo: el binario real de Ramal Desktop (motor en proceso, git real,
 * WebKitGTK en Linux) manejado con WebdriverIO y @wdio/tauri-service.
 *
 * Usa el driver embebido (tauri-plugin-wdio-webdriver), el unico que tambien
 * cubre macOS (ADR 0006). Solo esta en los builds con la feature `e2e`, que es
 * como lo compila `just desktop-e2e-native` antes de lanzar esto. Esa misma
 * feature deja que RAMAL_E2E_FOLDER elija la carpeta en lugar del dialogo
 * nativo, que WebDriver no puede manejar.
 */
const repoRoot = path.resolve(fileURLToPath(new URL('.', import.meta.url)), '../..');
const targetDir = process.env['CARGO_TARGET_DIR'] ?? path.join(repoRoot, 'target');
const binaryName = process.platform === 'win32' ? 'ramal-desktop.exe' : 'ramal-desktop';
const application = path.join(targetDir, 'debug', binaryName);

/** Variable que lee el selector de carpetas del build de E2E (src-tauri/src/bootstrap.rs). */
const E2E_FOLDER_ENV_VAR = 'RAMAL_E2E_FOLDER';

let fixture: FixtureRepo | undefined;

export const config: WebdriverIO.Config = {
  runner: 'local',
  specs: ['./wdio/specs/**/*.e2e.ts'],
  maxInstances: 1,
  capabilities: [
    {
      browserName: 'tauri',
      'tauri:options': { application },
    } as WebdriverIO.Capabilities,
  ],
  services: [['@wdio/tauri-service', { driverProvider: 'embedded' }]],
  framework: 'mocha',
  reporters: ['spec'],
  mochaOpts: { ui: 'bdd', timeout: 120_000 },
  logLevel: 'warn',
  waitforTimeout: 30_000,
  connectionRetryTimeout: 120_000,
  // El repositorio de prueba se crea antes de lanzar la app, que hereda el entorno.
  onPrepare: () => {
    fixture = createFixtureRepo();
    process.env[E2E_FOLDER_ENV_VAR] = fixture.path;
  },
  onComplete: () => {
    fixture?.dispose();
  },
  // El servicio intenta reenfocar la ventana activa antes de cada busqueda con
  // tauri-plugin-wdio y la API global de Tauri, que esta app no lleva
  // (withGlobalTauri apagado): cada intento esperaria 5 s para nada. Elegir la
  // ventana a mano le dice que no lo haga.
  before: async () => {
    const [handle] = await browser.getWindowHandles();
    if (handle !== undefined) await browser.switchToWindow(handle);
  },
};
