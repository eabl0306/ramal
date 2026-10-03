import * as assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';

import { PROTOCOL_VERSION } from '@git-rudder/bridge';
import * as vscode from 'vscode';

import type { HostProbe } from '../../src/extension/application/use-cases/host-probe';

const EXTENSION_ID = 'git-rudder.git-rudder';
const OPEN_GRAPH = 'gitRudder.openGraph';
/** Los commits del repositorio de prueba (scripts/fixture-repo.mjs), del mas nuevo al mas viejo. */
const SUMMARIES = ['Tercer commit', 'Segundo commit', 'Primer commit'];

async function probe(): Promise<HostProbe> {
  const extension = vscode.extensions.getExtension<HostProbe | undefined>(EXTENSION_ID);
  assert.ok(extension, `${EXTENSION_ID} is not installed`);
  const api = await extension.activate();
  assert.ok(api, 'the extension exposes its probe in test mode');
  return api;
}

/** Lo poco que se usa de la API de la extension integrada `vscode.git`. */
interface GitExtension {
  readonly enabled: boolean;
  getAPI(version: 1): { readonly git: { readonly path: string } };
}

/** El git que usa VS Code, segun su extension integrada. */
async function vscodeGitPath(): Promise<string> {
  const extension = vscode.extensions.getExtension<GitExtension>('vscode.git');
  assert.ok(extension, 'the built-in git extension is installed');
  const git = await extension.activate();
  assert.ok(git.enabled, 'the built-in git extension is enabled');
  return git.getAPI(1).git.path;
}

function workspacePath(): string {
  const folder = vscode.workspace.workspaceFolders?.[0];
  assert.ok(folder, 'the tests open the fixture repository as workspace');
  return folder.uri.fsPath;
}

/** Espera a que se cumpla una condicion, comprobando cada 100 ms. */
async function waitFor(condition: () => boolean, what: string, timeoutMs = 30_000): Promise<void> {
  const deadline = Date.now() + timeoutMs;
  while (!condition()) {
    if (Date.now() > deadline) assert.fail(`timed out waiting for ${what}`);
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
}

/**
 * Repite `call` mientras el motor no este (reiniciandose) y devuelve lo que
 * conteste. Cualquier otro error, como `repo_not_open`, falla el test.
 */
async function whenAvailable<T>(call: () => Promise<T>, timeoutMs = 30_000): Promise<T> {
  const deadline = Date.now() + timeoutMs;
  for (;;) {
    try {
      return await call();
    } catch (error) {
      const restarting = error instanceof Error && error.name === 'EngineUnavailableError';
      if (!restarting || Date.now() > deadline) throw error;
      await new Promise((resolve) => setTimeout(resolve, 200));
    }
  }
}

/** Mata con SIGKILL los sidecars que cuelgan de este extension host, como una caida. */
function killSidecar(): void {
  // -f mira la linea de comando entera: Linux corta el nombre del proceso a 15
  // caracteres y "git-rudder-vscode-engine" no cabe. El -P lo deja en los hijos
  // de este extension host, asi que no puede cazar nada ajeno.
  const output = execFileSync(
    'pgrep',
    ['-P', String(process.pid), '-f', 'git-rudder-vscode-engine'],
    { encoding: 'utf8' },
  );
  const pids = output.split('\n').filter((line) => line.trim() !== '');
  assert.ok(pids.length > 0, 'the engine sidecar is running');
  for (const pid of pids) process.kill(Number(pid), 'SIGKILL');
}

function graphTabOpen(): boolean {
  return vscode.window.tabGroups.all
    .flatMap((group) => group.tabs)
    .some(
      (tab) =>
        tab.input instanceof vscode.TabInputWebview &&
        tab.input.viewType.endsWith('gitRudder.graph'),
    );
}

suite('Git Rudder dentro de un VS Code de verdad', () => {
  test('se activa y registra el comando Git Rudder: Open Graph', async () => {
    await probe();
    const commands = await vscode.commands.getCommands(true);
    assert.ok(commands.includes(OPEN_GRAPH));
  });

  test('el sidecar contesta engine/info con el protocolo que trae la extension', async () => {
    const info = await (await probe()).call('engine/info', {});
    assert.equal(info.protocolVersion, PROTOCOL_VERSION);
    assert.equal(info.git.supported, true);
  });

  test('el sidecar usa el git de VS Code, no el primero del PATH', async () => {
    const info = await (await probe()).call('engine/info', {});
    const expected = await vscodeGitPath();
    // .vscode-test.mjs configura un git.path que no es el del PATH (salvo en
    // Windows): si VS Code lo respeta, el sidecar solo lo ve si el host se lo pasa.
    const configured = process.env['GIT_RUDDER_IT_GIT_PATH'] ?? '';
    if (configured !== '') assert.equal(expected, configured);
    assert.equal(info.git.path, expected);
  });

  test('el sidecar abre el repositorio de prueba y devuelve su grafo', async () => {
    const host = await probe();
    const handle = await host.call('repo/open', { path: workspacePath() });
    assert.equal(handle.headRef, 'main');
    const window = await host.call('graph/window', { repoId: handle.repoId, offset: 0, limit: 50 });
    assert.deepEqual(
      window.rows.map((row) => row.summary),
      SUMMARIES,
    );
    assert.equal(window.hasMore, false);
    const labels = window.rows.flatMap((row) => row.refs.map((ref) => ref.name));
    assert.deepEqual(labels.sort(), ['feature/graph', 'main']);
  });

  test('Git Rudder: Open Graph abre el panel y el webview pide el grafo al motor', async () => {
    const host = await probe();
    await vscode.commands.executeCommand(OPEN_GRAPH);
    await waitFor(graphTabOpen, 'the Git Rudder Graph tab');
    await waitFor(
      () =>
        host
          .routed()
          .some((entry) => entry.method === 'graph/window' && entry.outcome === 'answered'),
      'graph/window from the webview to be answered by the engine',
    );
    assert.ok(host.routed().every((entry) => entry.outcome === 'answered'));
  });

  test('si el sidecar se cae, el host lo reinicia y vuelve a abrir el repo del panel', async () => {
    const host = await probe();
    assert.ok(graphTabOpen(), 'the previous test left the graph panel open');
    const { repoId } = await host.call('repo/open', { path: workspacePath() });
    killSidecar();
    // Sin volver a abrirlo, el motor nuevo contesta repo_not_open y el panel muere.
    const window = await whenAvailable(() =>
      host.call('graph/window', { repoId, offset: 0, limit: 50 }),
    );
    assert.deepEqual(
      window.rows.map((row) => row.summary),
      SUMMARIES,
    );
  });
});
