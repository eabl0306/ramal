export interface EngineArgsInput {
  /** El git que usa VS Code (API de `vscode.git`), si lo hay. */
  readonly gitPath: string | undefined;
  /** Carpeta de datos de la extension (`globalStorageUri`), si la hay. */
  readonly dataDir?: string | undefined;
  /** Filtro de log del sidecar. */
  readonly logLevel: string;
}

/**
 * Argumentos con los que se lanza `git-rudder-vscode-engine` (apps/vscode-engine,
 * args.rs). El git va primero porque es lo que el usuario ya le dijo a su editor
 * que use (ADR 0009); sin el, el sidecar busca el primero del `PATH`. Una ruta
 * vacia no se pasa: el sidecar la tomaria por un git que no existe. Cada ruta va
 * en su propio argumento y no como `--git-path=...`, que el sidecar lee como
 * texto y estropearia una ruta que no sea UTF-8.
 *
 * `--data-dir` es donde el motor recuerda en que repositorios confio el usuario
 * (ADR 0020). Es el almacen global de la extension: queda en el dispositivo y
 * VS Code no lo sincroniza. Sin el, la confianza se olvida al cerrar.
 */
export function engineArgs({ gitPath, dataDir, logLevel }: EngineArgsInput): string[] {
  return [
    ...pathArg('--git-path', gitPath),
    ...pathArg('--data-dir', dataDir),
    '--log-level',
    logLevel,
  ];
}

function pathArg(flag: string, path: string | undefined): string[] {
  return path === undefined || path === '' ? [] : [flag, path];
}
