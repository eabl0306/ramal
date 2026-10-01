/**
 * Lo que el webview usa de la API que VS Code inyecta. `acquireVsCodeApi` es un
 * global que solo se puede llamar una vez por pagina; los tests de Playwright
 * lo sustituyen por uno falso.
 */
export interface VsCodeApi {
  postMessage(message: unknown): void;
}

/** De donde llegan los mensajes del host: `window` en el webview. */
export interface MessageSource {
  addEventListener(type: 'message', listener: (event: MessageEvent) => void): void;
  removeEventListener(type: 'message', listener: (event: MessageEvent) => void): void;
}

declare global {
  function acquireVsCodeApi(): VsCodeApi;
}

/** La API de VS Code de esta pagina. */
export function vsCodeApi(): VsCodeApi {
  return acquireVsCodeApi();
}
