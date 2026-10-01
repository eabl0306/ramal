/** El portapapeles del sistema, para los comandos de instalacion. */
export interface Clipboard {
  /** true si se copio; el navegador puede negarlo sin permiso o sin HTTPS. */
  readonly copy: (text: string) => Promise<boolean>;
}
