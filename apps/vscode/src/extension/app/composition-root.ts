import * as vscode from 'vscode';

import type { Logger } from '../application/ports/logger';
import { type HostProbe, createHostProbe } from '../application/use-cases/host-probe';
import { type OpenGraphDeps, openGraph } from '../application/use-cases/open-graph';
import {
  type WebviewRouter,
  createWebviewRouter,
} from '../application/use-cases/route-webview-message';
import { type HostRuntime, startup } from '../application/use-cases/startup';
import { type HostMode, resolveEngineBinary } from '../infrastructure/engine/engine-binary';
import { createSidecarLauncher } from '../infrastructure/engine/sidecar-launcher';
import { systemTimer } from '../infrastructure/system-timer';
import { vscodeHostRequests } from '../infrastructure/vscode/clipboard-host-requests';
import { hostMode, readEngineSettings } from '../infrastructure/vscode/engine-settings';
import {
  vscodeDialogs,
  vscodeNotifications,
  workspaceRepositories,
} from '../infrastructure/vscode/native-ui';
import { createOutputLogger } from '../infrastructure/vscode/output-logger';
import { registerCommands, registerWelcomeView } from '../infrastructure/vscode/registrations';
import { VsCodeWebviewPanels } from '../infrastructure/vscode/webview-panels';
import { devServerOrigin } from '../infrastructure/webview/webview-html';

/**
 * Composition root del extension host: el unico sitio que construye los
 * adaptadores y los casos de uso y los cablea. Todo lo que crea queda en
 * `context.subscriptions`, asi VS Code lo libera al desactivar la extension (el
 * motor incluido: cerrar su stdin lo termina).
 *
 * Devuelve la sonda de los tests de integracion solo con `ExtensionMode.Test`.
 */
export function composeExtension(context: vscode.ExtensionContext): HostProbe | undefined {
  const hostLog = vscode.window.createOutputChannel('Ramal', { log: true });
  // El sidecar ya escribe su log con hora y nivel (tracing): va tal cual.
  const engineLog = vscode.window.createOutputChannel('Ramal Engine');
  const logger = createOutputLogger(hostLog);
  const mode = hostMode(context.extensionMode);
  const runtime = startup({
    language: vscode.env.language,
    launcher: engineLauncher(context, mode, logger, engineLog),
    timer: systemTimer,
    notifications: vscodeNotifications,
    logger,
  });
  const router = createWebviewRouter({
    engine: runtime.engine,
    ids: runtime.ids,
    hostRequests: vscodeHostRequests,
    logger,
    locale: runtime.locale,
    protocolVersion: runtime.protocolVersion,
  });
  const panels = webviewPanels(context, mode, runtime, router, logger);
  const graphDeps = openGraphDeps(runtime, panels, logger);
  const commands = registerCommands({ openGraph: () => openGraph(graphDeps) }, logger);
  context.subscriptions.push(hostLog, engineLog, panels, registerWelcomeView(), ...commands, {
    dispose: () => {
      router.dispose();
      runtime.engine.stop();
    },
  });
  return mode === 'test' ? createHostProbe(router, runtime.engine, runtime.ids) : undefined;
}

/** El lanzador del sidecar con el binario que toca segun el modo. */
function engineLauncher(
  context: vscode.ExtensionContext,
  mode: HostMode,
  logger: Logger,
  engineLog: vscode.OutputChannel,
): ReturnType<typeof createSidecarLauncher> {
  const settings = readEngineSettings();
  const binaryPath = resolveEngineBinary({
    extensionPath: context.extensionPath,
    mode,
    platform: process.platform,
    configuredPath: settings.path,
    environmentPath: process.env['RAMAL_ENGINE_PATH'],
  });
  logger.info(`engine binary: ${binaryPath}`);
  return createSidecarLauncher({
    binaryPath,
    args: ['--log-level', settings.logLevel],
    logger,
    onEngineLog: (line) => {
      engineLog.appendLine(line);
    },
  });
}

/** Los paneles; el servidor de Vite solo cuenta en modo desarrollo. */
function webviewPanels(
  context: vscode.ExtensionContext,
  mode: HostMode,
  runtime: HostRuntime,
  router: WebviewRouter,
  logger: Logger,
): VsCodeWebviewPanels {
  const devServer =
    mode === 'development' ? devServerOrigin(process.env['RAMAL_WEBVIEW_DEV_SERVER']) : undefined;
  return new VsCodeWebviewPanels({
    extensionUri: context.extensionUri,
    locale: runtime.locale,
    logger,
    router,
    devServer,
  });
}

function openGraphDeps(
  runtime: HostRuntime,
  panels: VsCodeWebviewPanels,
  logger: Logger,
): OpenGraphDeps {
  return {
    engine: runtime.engine,
    ids: runtime.ids,
    repositories: workspaceRepositories,
    dialogs: vscodeDialogs,
    notifications: vscodeNotifications,
    panels,
    translate: runtime.translate,
    logger,
  };
}
