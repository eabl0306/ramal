import { PROTOCOL_VERSION } from '@ramal/bridge';
import { createTranslator, type Locale, resolveLocale, type Translate } from '@ramal/i18n';

import type { EngineLauncher } from '../ports/engine-launcher';
import type { Logger } from '../ports/logger';
import type { Notifications } from '../ports/notifications';
import type { Timer } from '../ports/timer';
import { createRequestIds, type RequestIds } from '../request-ids';

import { checkEngineProtocol } from './check-engine-protocol';
import { type SupervisedEngine, superviseEngine } from './supervise-engine';

export interface StartupDeps {
  /** `vscode.env.language`: el idioma de VS Code. */
  readonly language: string;
  readonly launcher: EngineLauncher;
  readonly timer: Timer;
  readonly notifications: Notifications;
  readonly logger: Logger;
}

/** Lo que el resto del host necesita: idioma, textos, ids y el motor supervisado. */
export interface HostRuntime {
  readonly locale: Locale;
  readonly translate: Translate;
  readonly ids: RequestIds;
  readonly engine: SupervisedEngine;
  readonly protocolVersion: number;
}

/**
 * Arranque del host (architecture.md 4.4 y 7.5): fija el idioma con la regla
 * comun sobre el de VS Code y prepara el motor supervisado, que compara
 * `PROTOCOL_VERSION` cada vez que arranca un proceso. El sidecar no se lanza
 * aqui sino con la primera peticion: activar la extension no cuesta un proceso.
 */
export function startup(deps: StartupDeps): HostRuntime {
  const locale = resolveLocale(deps.language);
  const translate = createTranslator(locale);
  const ids = createRequestIds();
  const engine = superviseEngine({
    launcher: deps.launcher,
    timer: deps.timer,
    logger: deps.logger,
    handshake: (connection) => checkEngineProtocol(connection, ids, PROTOCOL_VERSION),
    onGiveUp: (retry) => {
      void announceGiveUp(deps.notifications, translate, retry);
    },
    onProtocolMismatch: (mismatch) => {
      const params = { engine: mismatch.engineVersion, expected: mismatch.expected };
      void deps.notifications.showError(translate('vscode.host.protocolMismatch', params));
    },
  });
  deps.logger.info(`Ramal host ready (locale ${locale}, protocol ${String(PROTOCOL_VERSION)})`);
  return { locale, translate, ids, engine, protocolVersion: PROTOCOL_VERSION };
}

/** El aviso de "deje de intentarlo", con su boton de reintentar. */
async function announceGiveUp(
  notifications: Notifications,
  translate: Translate,
  retry: () => void,
): Promise<void> {
  const retryLabel = translate('vscode.host.retry');
  const choice = await notifications.showError(translate('vscode.host.engineStopped'), retryLabel);
  if (choice === retryLabel) retry();
}
