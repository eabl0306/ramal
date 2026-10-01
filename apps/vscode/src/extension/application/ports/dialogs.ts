/** Una opcion de una lista nativa. */
export interface PickItem<T> {
  readonly label: string;
  readonly description?: string;
  readonly value: T;
}

/** Dialogos nativos de VS Code (QuickPick, InputBox y avisos modales). */
export interface Dialogs {
  /** Devuelve el valor elegido, o `undefined` si se cancela. */
  pick<T>(items: readonly PickItem<T>[], placeholder: string): Promise<T | undefined>;
}
