/**
 * Avisos nativos de VS Code. Los textos llegan ya traducidos: la aplicacion es
 * la unica capa que usa @git-rudder/i18n.
 */
export interface Notifications {
  showInfo(message: string): void;
  /** Ensena un error con botones y devuelve el que se pulso, o `undefined`. */
  showError(message: string, ...actions: string[]): Promise<string | undefined>;
}
