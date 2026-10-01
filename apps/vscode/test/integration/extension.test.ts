import * as assert from 'node:assert/strict';

import { PROTOCOL_VERSION } from '@ramal/bridge';
import * as vscode from 'vscode';

import type { HostProbe } from '../../src/extension/application/use-cases/host-probe';

const EXTENSION_ID = 'ramal.ramal';
const OPEN_GRAPH = 'ramal.openGraph';
/** Los commits del repositorio de prueba (scripts/fixture-repo.mjs), del mas nuevo al mas viejo. */
const SUMMARIES = ['Tercer commit', 'Segundo commit', 'Primer commit'];

async function probe(): Promise<HostProbe> {
  const extension = vscode.extensions.getExtension<HostProbe | undefined>(EXTENSION_ID);
  assert.ok(extension, `${EXTENSION_ID} is not installed`);
  const api = await extension.activate();
  assert.ok(api, 'the extension exposes its probe in test mode');
  return api;
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

function graphTabOpen(): boolean {
  return vscode.window.tabGroups.all
    .flatMap((group) => group.tabs)
    .some(
      (tab) =>
        tab.input instanceof vscode.TabInputWebview && tab.input.viewType.endsWith('ramal.graph'),
    );
}

suite('Ramal dentro de un VS Code de verdad', () => {
  test('se activa y registra el comando Ramal: Open Graph', async () => {
    await probe();
    const commands = await vscode.commands.getCommands(true);
    assert.ok(commands.includes(OPEN_GRAPH));
  });

  test('el sidecar contesta engine/info con el protocolo que trae la extension', async () => {
    const info = await (await probe()).call('engine/info', {});
    assert.equal(info.protocolVersion, PROTOCOL_VERSION);
    assert.equal(info.git.supported, true);
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

  test('Ramal: Open Graph abre el panel y el webview pide el grafo al motor', async () => {
    const host = await probe();
    await vscode.commands.executeCommand(OPEN_GRAPH);
    await waitFor(graphTabOpen, 'the Ramal Graph tab');
    await waitFor(
      () =>
        host
          .routed()
          .some((entry) => entry.method === 'graph/window' && entry.outcome === 'answered'),
      'graph/window from the webview to be answered by the engine',
    );
    assert.ok(host.routed().every((entry) => entry.outcome === 'answered'));
  });
});
