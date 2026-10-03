import { describe, expect, it, vi } from 'vitest';

import { createBrowserClipboard } from './browser-clipboard';

describe('createBrowserClipboard', () => {
  it('copia el texto y lo confirma', async () => {
    const writeText = vi.fn(() => Promise.resolve());
    const clipboard = createBrowserClipboard(() => ({ writeText }));

    await expect(clipboard.copy('brew install --cask git-rudder')).resolves.toBe(true);
    expect(writeText).toHaveBeenCalledWith('brew install --cask git-rudder');
  });

  it('contesta false si el navegador lo niega', async () => {
    const clipboard = createBrowserClipboard(() => ({
      writeText: () => Promise.reject(new Error('NotAllowedError')),
    }));

    await expect(clipboard.copy('x')).resolves.toBe(false);
  });

  it('contesta false si no hay portapapeles', async () => {
    await expect(createBrowserClipboard(() => undefined).copy('x')).resolves.toBe(false);
  });
});
