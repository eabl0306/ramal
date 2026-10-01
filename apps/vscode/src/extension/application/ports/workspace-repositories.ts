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
}
