import { describe, expect, it } from 'vitest';

import { allDeclarations, blockWith, oklchNumbers, parseBlocks, readCss } from '../test/css';
import { DENSITIES, DENSITY_TOKENS } from '../tokens/densities';
import { COLOR_TOKENS, SHAPE_TOKENS, SHARED_TOKENS } from '../tokens/tokens';

const base = parseBlocks(readCss('base.css'));
const gitRudder = parseBlocks(readCss('git-rudder.css'));
const vscode = parseBlocks(readCss('vscode.css'));

/** La tabla del README del handoff: [L, C, H] de cada token, oscuro y claro. */
const HANDOFF: Readonly<Record<string, readonly [number[], number[]]>> = {
  '--r-bg': [
    [0.185, 0.008, 230],
    [0.965, 0.004, 230],
  ],
  '--r-panel': [
    [0.21, 0.009, 230],
    [0.985, 0.003, 230],
  ],
  '--r-surface': [
    [0.24, 0.01, 230],
    [1, 0, 0],
  ],
  '--r-surface-hover': [
    [0.285, 0.012, 230],
    [0.94, 0.006, 230],
  ],
  '--r-border': [
    [0.315, 0.012, 230],
    [0.89, 0.008, 230],
  ],
  '--r-text': [
    [0.95, 0.005, 230],
    [0.24, 0.012, 230],
  ],
  '--r-muted': [
    [0.76, 0.012, 230],
    [0.44, 0.012, 230],
  ],
  '--r-faint': [
    [0.62, 0.012, 230],
    [0.54, 0.01, 230],
  ],
  '--r-accent': [
    [0.74, 0.12, 190],
    [0.54, 0.1, 190],
  ],
  '--r-accent-ink': [
    [0.2, 0.03, 190],
    [0.99, 0.005, 190],
  ],
  '--r-ok': [
    [0.76, 0.15, 150],
    [0.52, 0.13, 150],
  ],
  '--r-warn': [
    [0.8, 0.14, 80],
    [0.58, 0.13, 70],
  ],
  '--r-danger': [
    [0.7, 0.17, 25],
    [0.55, 0.18, 25],
  ],
  '--r-info': [
    [0.74, 0.12, 245],
    [0.52, 0.13, 250],
  ],
};

const LANES = [
  [0.72, 0.15, 190],
  [0.68, 0.2, 300],
  [0.76, 0.16, 60],
  [0.68, 0.21, 355],
  [0.74, 0.18, 140],
  [0.7, 0.16, 250],
];

function value(block: { declarations: ReadonlyMap<string, string> }, token: string): string {
  return block.declarations.get(token) ?? '';
}

describe('git-rudder.css', () => {
  const dark = blockWith(gitRudder, "[data-theme='dark']");
  const light = blockWith(gitRudder, "[data-theme='light']");

  it('da valor a todos los tokens entre sus bloques y base.css', () => {
    const defined = new Set([...allDeclarations(gitRudder), ...allDeclarations(base)]);
    const missing = [...COLOR_TOKENS, ...SHAPE_TOKENS, ...SHARED_TOKENS].filter(
      (token) => !defined.has(token),
    );
    expect(missing).toEqual([]);
  });

  it('usa los oklch exactos del handoff en oscuro y en claro', () => {
    for (const [token, [darkValue, lightValue]] of Object.entries(HANDOFF)) {
      expect(oklchNumbers(value(dark, token)), `${token} oscuro`).toEqual([...darkValue, 1]);
      expect(oklchNumbers(value(light, token)), `${token} claro`).toEqual([...lightValue, 1]);
    }
  });

  it('el acento suave es el acento al 14% en oscuro y al 10% en claro', () => {
    expect(oklchNumbers(value(dark, '--r-accent-soft'))).toEqual([0.74, 0.12, 190, 0.14]);
    expect(oklchNumbers(value(light, '--r-accent-soft'))).toEqual([0.54, 0.1, 190, 0.1]);
  });

  it('el escritorio usa 13 px y la web 14 px, con su claro mas luminoso', () => {
    const shared = gitRudder.find((block) => block.declarations.has('--r-font-size-base'));
    expect(shared?.declarations.get('--r-font-size-base')).toBe('13px');
    expect(value(blockWith(gitRudder, "[data-host='web']"), '--r-font-size-base')).toBe('14px');
    const webLight = blockWith(gitRudder, "[data-host='web'][data-theme='light']");
    expect(oklchNumbers(value(webLight, '--r-bg'))).toEqual([0.975, 0.004, 230, 1]);
  });
});

describe('base.css', () => {
  it('tiene los seis carriles del diseno, iguales en todos los temas', () => {
    const root = blockWith(base, ':root');
    LANES.forEach((lane, index) => {
      expect(oklchNumbers(value(root, `--r-lane-${String(index)}`))).toEqual([...lane, 1]);
    });
  });

  it('las densidades del CSS coinciden con DENSITIES', () => {
    const comfortable = blockWith(base, "[data-density='comfortable']");
    const compact = blockWith(base, "[data-density='compact']");
    for (const [metric, token] of Object.entries(DENSITY_TOKENS)) {
      const key = metric as keyof typeof DENSITY_TOKENS;
      expect(value(comfortable, token)).toBe(`${String(DENSITIES.comfortable[key])}px`);
      expect(value(compact, token)).toBe(`${String(DENSITIES.compact[key])}px`);
    }
  });
});

describe('vscode.css', () => {
  const root = blockWith(vscode, ':root');

  it('cada token de color apunta a una variable de VS Code', () => {
    const offenders = COLOR_TOKENS.filter((token) => !value(root, token).includes('var(--vscode-'));
    expect(offenders).toEqual([]);
  });

  it('la tipografia es la del editor', () => {
    expect(value(root, '--r-font-ui')).toBe('var(--vscode-font-family)');
    expect(value(root, '--r-font-code')).toBe('var(--vscode-editor-font-family)');
    expect(value(root, '--r-font-size-base')).toBe('var(--vscode-font-size)');
  });

  it('el alto contraste oscuro y claro dibuja bordes con contrastBorder', () => {
    const contrast = blockWith(vscode, 'body.vscode-high-contrast');
    expect(contrast.selector).toContain('body.vscode-high-contrast-light');
    expect(value(contrast, '--r-hc-border')).toBe('var(--vscode-contrastBorder)');
    expect(value(contrast, '--r-border')).toBe('var(--vscode-contrastBorder)');
    expect(value(contrast, '--r-hc-active-border')).toContain('var(--vscode-contrastActiveBorder');
  });

  it('la fila elegida con foco usa la seleccion activa con su tinta y sin foco la inactiva', () => {
    // El --sel del prototipo es list.activeSelectionBackground; --sel2, la inactiva.
    expect(value(root, '--r-selection')).toBe('var(--vscode-list-activeSelectionBackground)');
    expect(value(root, '--r-selection-ink')).toContain(
      'var(--vscode-list-activeSelectionForeground',
    );
    expect(value(root, '--r-accent-soft')).toBe('var(--vscode-list-inactiveSelectionBackground)');
    const contrast = blockWith(vscode, 'body.vscode-high-contrast');
    expect(value(contrast, '--r-selection')).toBe('transparent');
  });
});

describe('git-rudder.css: seleccion', () => {
  it('en escritorio y web la seleccion con foco es el acento suave de siempre', () => {
    const block = gitRudder.find((candidate) => candidate.declarations.has('--r-selection'));
    expect(block?.declarations.get('--r-selection')).toBe('var(--r-accent-soft)');
    expect(block?.declarations.get('--r-selection-ink')).toBe('var(--r-text)');
  });
});
