import type { Clipboard } from '../../application/ports/clipboard';

interface ClipboardLike {
  readonly writeText: (text: string) => Promise<void>;
}

const browserClipboard = (): ClipboardLike | undefined =>
  typeof navigator === 'undefined' ? undefined : navigator.clipboard;

/**
 * El portapapeles del navegador. Puede no existir (HTTP sin TLS fuera de
 * localhost) o negarse sin permiso: se contesta false y la interfaz no presume
 * de haber copiado.
 */
export function createBrowserClipboard(
  source: () => ClipboardLike | undefined = browserClipboard,
): Clipboard {
  return {
    copy: async (text) => {
      const clipboard = source();
      if (clipboard === undefined) return false;
      try {
        await clipboard.writeText(text);
        return true;
      } catch {
        return false;
      }
    },
  };
}
