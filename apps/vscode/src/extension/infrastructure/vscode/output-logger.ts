import type * as vscode from 'vscode';

import type { Logger } from '../../application/ports/logger';

/** El log del host en el canal de salida "Git Rudder", con niveles de VS Code. */
export function createOutputLogger(channel: vscode.LogOutputChannel): Logger {
  return {
    debug: (message) => {
      channel.debug(message);
    },
    info: (message) => {
      channel.info(message);
    },
    warn: (message) => {
      channel.warn(message);
    },
    error: (message) => {
      channel.error(message);
    },
  };
}
