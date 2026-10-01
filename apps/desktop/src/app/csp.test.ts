import { describe, expect, it } from 'vitest';

import tauriConfig from '../../src-tauri/tauri.conf.json';

/**
 * Lo que la CSP de tauri.conf.json garantiza (architecture.md 6.8). El E2E de
 * Playwright sirve el build con esta misma politica y falla con cualquier
 * violacion; aqui se fija la forma para que nadie la relaje sin verlo.
 */
describe('CSP de produccion', () => {
  const { csp } = tauriConfig.app.security;

  it('el webview solo habla con el IPC de Tauri', () => {
    expect(csp['connect-src']).toBe('ipc: http://ipc.localhost');
    expect(csp['default-src']).toBe("'none'");
    expect(csp['script-src']).toBe("'self'");
  });

  it('sin estilos en linea salvo el de React Aria, permitido por su hash', () => {
    // react-aria-components 1.21 mete un <style> con el touch-action de lo pulsable.
    // Se permite ese texto exacto por hash, nunca 'unsafe-inline'; si React Aria lo
    // cambia, el E2E ve la violacion de la CSP.
    expect(csp['style-src']).toMatch(/^'self' 'sha256-[A-Za-z0-9+/]+=*'$/u);
  });

  it('el prototipo congelado y sin API global de Tauri', () => {
    expect(tauriConfig.app.security.freezePrototype).toBe(true);
    expect(tauriConfig.app.withGlobalTauri).toBe(false);
  });
});
