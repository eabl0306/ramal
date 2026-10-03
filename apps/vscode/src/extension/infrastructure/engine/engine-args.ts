export interface EngineArgsInput {
  /** El git que usa VS Code (API de `vscode.git`), si lo hay. */
  readonly gitPath: string | undefined;
  /** Filtro de log del sidecar. */
  readonly logLevel: string;
}

/**
 * Argumentos con los que se lanza `git-rudder-vscode-engine` (apps/vscode-engine,
 * args.rs). El git va primero porque es lo que el usuario ya le dijo a su editor
 * que use (ADR 0009); sin el, el sidecar busca el primero del `PATH`. Una ruta
 * vacia no se pasa: el sidecar la tomaria por un git que no existe. La ruta va
 * en su propio argumento y no como `--git-path=...`, que el sidecar lee como
 * texto y estropearia una ruta que no sea UTF-8.
 */
export function engineArgs({ gitPath, logLevel }: EngineArgsInput): string[] {
  const git = gitPath === undefined || gitPath === '' ? [] : ['--git-path', gitPath];
  return [...git, '--log-level', logLevel];
}
