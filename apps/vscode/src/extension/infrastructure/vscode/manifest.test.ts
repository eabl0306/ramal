import { readFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { LOCALES, createTranslator } from '@git-rudder/i18n';
import { describe, expect, it } from 'vitest';

/**
 * El manifiesto de la extension manda sobre lo que la landing promete. Existe
 * por un fallo real: la tarjeta de VS Code decia "1.90 o posterior" mientras
 * engines.vscode pedia ^1.138.0, y VS Code se niega a instalar una extension
 * que pide un motor mas nuevo que el suyo. Quien suba engines.vscode ve este
 * test en rojo hasta que la web diga lo mismo.
 */

interface Manifest {
  readonly engines: { readonly vscode: string };
  readonly capabilities: { readonly virtualWorkspaces: boolean };
}

/** Ruta construida a mano: Vite reescribe `new URL(..., import.meta.url)` como un asset. */
function manifest(): Manifest {
  const here = path.dirname(fileURLToPath(import.meta.url));
  const file = path.join(here, '..', '..', '..', '..', 'package.json');
  return JSON.parse(readFileSync(file, 'utf8')) as Manifest;
}

/** "^1.140.0" es "1.140": la landing ensena mayor y menor. */
function minimumVersion(range: string): string {
  const match = /^\^?(\d+)\.(\d+)\.\d+$/u.exec(range);
  if (match === null) throw new Error(`engines.vscode no es una version simple: ${range}`);
  return `${match[1] ?? ''}.${match[2] ?? ''}`;
}

describe('package.json de la extension frente a la landing', () => {
  const { engines, capabilities } = manifest();

  it('la tarjeta de VS Code pide la misma version minima que engines.vscode', () => {
    const minimum = minimumVersion(engines.vscode);
    for (const locale of LOCALES) {
      const requirement = createTranslator(locale)('web.vscode.requirement');
      expect(requirement, locale).toMatch(
        new RegExp(`^VS Code ${minimum.replace('.', '\\.')} `, 'u'),
      );
    }
  });

  it('sin espacios de trabajo virtuales la tarjeta avisa de que no hay vscode.dev', () => {
    expect(capabilities.virtualWorkspaces).toBe(false);
    for (const locale of LOCALES) {
      expect(createTranslator(locale)('web.vscode.requirement'), locale).toContain('vscode.dev');
    }
  });
});
