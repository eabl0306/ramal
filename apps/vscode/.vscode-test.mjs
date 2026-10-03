// Tests de integracion con @vscode/test-cli: descargan un VS Code de verdad (en
// .vscode-test/), cargan la extension en modo test y corren
// out/test/integration con mocha. Sin pantalla, se lanzan con `xvfb-run -a`
// (lo hace `just vscode-test-integration`).
import { existsSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { defineConfig } from '@vscode/test-cli';

import { createGitPathProfile } from './scripts/fixture-git-path.mjs';
import { createFixtureRepo } from './scripts/fixture-repo.mjs';

const root = path.dirname(fileURLToPath(import.meta.url));

/**
 * El sidecar que se prueba: el de GIT_RUDDER_ENGINE_PATH o el que compilo cargo, con
 * release primero porque es lo que construye `just vscode-engine-build`.
 */
function engineBinary() {
  const explicit = process.env['GIT_RUDDER_ENGINE_PATH'];
  if (explicit !== undefined && explicit !== '') return explicit;
  const name =
    process.platform === 'win32' ? 'git-rudder-vscode-engine.exe' : 'git-rudder-vscode-engine';
  const found = ['release', 'debug']
    .map((profile) => path.join(root, '..', '..', 'target', profile, name))
    .find((candidate) => existsSync(candidate));
  if (found === undefined) {
    throw new Error('no git-rudder-vscode-engine binary: run `just vscode-engine-build` first');
  }
  return found;
}

// Perfil propio con un `git.path` distinto del git del PATH (ver el script).
// Pasar --user-data-dir evita que @vscode/test-electron use el suyo.
const profile = createGitPathProfile();

export default defineConfig({
  label: 'integration',
  files: 'out/test/integration/**/*.test.cjs',
  // La minima que declara engines.vscode: si funciona aqui, funciona en las nuevas.
  version: '1.138.0',
  extensionDevelopmentPath: root,
  workspaceFolder: createFixtureRepo(),
  launchArgs: [
    `--user-data-dir=${profile.userDataDir}`,
    '--disable-extensions',
    '--disable-workspace-trust',
    '--skip-welcome',
    '--skip-release-notes',
  ],
  env: { GIT_RUDDER_ENGINE_PATH: engineBinary(), GIT_RUDDER_IT_GIT_PATH: profile.gitPath },
  mocha: { ui: 'tdd', timeout: 60_000 },
});
