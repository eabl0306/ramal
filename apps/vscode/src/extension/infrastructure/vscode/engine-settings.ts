import * as vscode from 'vscode';

import type { HostMode } from '../engine/engine-binary';

/** Niveles de log que acepta el sidecar en `--log-level`. */
const LOG_LEVELS = ['error', 'warn', 'info', 'debug', 'trace'] as const;

type LogLevel = (typeof LOG_LEVELS)[number];

export interface EngineSettings {
  /** `gitRudder.engine.path` del usuario; el de workspace no se lee nunca. */
  readonly path: string | undefined;
  readonly logLevel: LogLevel;
}

function isLogLevel(value: unknown): value is LogLevel {
  return LOG_LEVELS.some((level) => level === value);
}

/**
 * Los ajustes del motor. La ruta se lee solo del ambito global: aunque el
 * manifiesto la declare de aplicacion, leer `globalValue` deja claro que una
 * carpeta abierta no puede elegir que binario se ejecuta. El nivel de log se
 * comprueba contra la lista antes de pasarlo como argumento.
 */
export function readEngineSettings(): EngineSettings {
  const config = vscode.workspace.getConfiguration('gitRudder.engine');
  const path = config.inspect<string>('path')?.globalValue;
  const level = config.get<unknown>('logLevel');
  return { path, logLevel: isLogLevel(level) ? level : 'info' };
}

/** El modo de ejecucion de la extension en los terminos del resto del host. */
export function hostMode(mode: vscode.ExtensionMode): HostMode {
  switch (mode) {
    case vscode.ExtensionMode.Development:
      return 'development';
    case vscode.ExtensionMode.Test:
      return 'test';
    case vscode.ExtensionMode.Production:
      return 'production';
  }
}
