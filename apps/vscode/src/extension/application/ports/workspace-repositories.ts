/** Una carpeta del workspace que puede ser un repositorio. */
export interface WorkspaceFolderInfo {
  readonly name: string;
  /** Ruta absoluta en el sistema de ficheros donde corre el host. */
  readonly path: string;
}

/**
 * Las carpetas del workspace abierto. La ruta solo la ve el host: se la pasa al
 * motor en `repo/open` y el webview recibe el `repoId`, nunca la ruta.
 */
export interface WorkspaceRepositories {
  list(): readonly WorkspaceFolderInfo[];
  /**
   * El usuario ya confio en esta ruta: el workspace es de confianza (Workspace
   * Trust de VS Code) y la ruta esta dentro de una de sus carpetas. Fuera de
   * ellas decide el motor con lo que encuentra (ADR 0020).
   */
  vouches(path: string): boolean;
}
