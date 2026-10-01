import { describe, expect, it } from 'vitest';

import { DELETED_FILE, I18N_FILE, LABELS, SESSION_FILE } from '../test/conflict-fixtures';

import { type ConflictOutput, buildOutput, hunkMarkerText } from './build-output';
import { resolveEmpty, takeAll, toggleBlock, toggleLine } from './choices';
import { type ConflictFile, initResolution } from './model';

/** Un fichero con un solo bloque entre dos lineas de contexto, resuelto tomando A y B enteros. */
function bothBlocks(ours: readonly string[], theirs: readonly string[]): ConflictOutput {
  const file: ConflictFile = {
    path: 'ejemplo.txt',
    segments: [
      { kind: 'context', lines: ['<antes>'] },
      { kind: 'hunk', ours, theirs },
      { kind: 'context', lines: ['<despues>'] },
    ],
  };
  let resolution = toggleBlock(file, initResolution(file), 0, 'ours');
  resolution = toggleBlock(file, resolution, 0, 'theirs');
  return buildOutput(file, resolution, LABELS);
}

describe('buildOutput', () => {
  it('sin elecciones deja los marcadores en el texto y un hueco sin numero', () => {
    const output = buildOutput(I18N_FILE, initResolution(I18N_FILE), LABELS);
    expect(output.text).toBe(
      [
        '{',
        '  "graph.zoom": "Zoom",',
        '  "graph.reset": "Restablecer vista",',
        '<<<<<<< feature/graph-ui',
        '  "auth.expired": "Sesión caducada",',
        '=======',
        '  "auth.expired": "Tu sesión ha expirado. Vuelve a iniciar sesión.",',
        '  "auth.reauth": "Iniciar sesión de nuevo",',
        '>>>>>>> hotfix/login',
        '  "auth.retry": "Reintentar"',
        '}',
      ].join('\n'),
    );
    expect(output.unresolved).toBe(1);
    expect(output.lines[3]).toEqual({ kind: 'unresolved', hunk: 0 });
    expect(output.lines[4]).toEqual({
      kind: 'context',
      text: '  "auth.retry": "Reintentar"',
      number: 4,
    });
  });

  it('reproduce el ejemplo del prototipo: linea de A y despues linea de B', () => {
    const picked = toggleLine(toggleLine(initResolution(I18N_FILE), 0, 'ours', 0), 0, 'theirs', 1);
    const output = buildOutput(I18N_FILE, picked, LABELS);
    expect(output.text).toBe(
      [
        '{',
        '  "graph.zoom": "Zoom",',
        '  "graph.reset": "Restablecer vista",',
        '  "auth.expired": "Sesión caducada",',
        '  "auth.reauth": "Iniciar sesión de nuevo",',
        '  "auth.retry": "Reintentar"',
        '}',
      ].join('\n'),
    );
    expect(output.lines[3]).toEqual({
      kind: 'picked',
      side: 'ours',
      hunk: 0,
      line: 0,
      text: '  "auth.expired": "Sesión caducada",',
      number: 4,
    });
    expect(output.unresolved).toBe(0);
  });

  it('reproduce el segundo ejemplo: todo de B y luego retoques en el bloque 2', () => {
    let resolution = takeAll(SESSION_FILE, initResolution(SESSION_FILE), 'theirs');
    resolution = toggleLine(resolution, 1, 'ours', 1);
    resolution = toggleLine(resolution, 1, 'theirs', 0);
    const lines = buildOutput(SESSION_FILE, resolution, LABELS).text.split('\n');
    const start = lines.indexOf('  const res = await api(acc).post("/oauth/refresh", {');
    expect(lines.slice(start + 1, start + 3)).toEqual([
      '    timeout: 5000,',
      '    scope: acc.scopes.join(" "),',
    ]);
    expect(lines).not.toContain('    refresh_token: token.refresh,');
  });

  it('una linea identica elegida en los dos lados se queda dos veces y se avisa', () => {
    let resolution = toggleBlock(SESSION_FILE, initResolution(SESSION_FILE), 1, 'ours');
    resolution = toggleBlock(SESSION_FILE, resolution, 1, 'theirs');
    const output = buildOutput(SESSION_FILE, resolution, LABELS);
    const occurrences = output.text
      .split('\n')
      .filter((line) => line === '    refresh_token: token.refresh,');
    expect(occurrences).toHaveLength(2);
    expect(output.duplicates).toBe(1);
  });

  it('tomar los dos bloques no borra el cierre ni la linea en blanco del segundo', () => {
    const ours = ['  fun a() {', '    return 1', '  }', ''];
    const theirs = ['  fun b() {', '    return 2', '  }', ''];
    const output = bothBlocks(ours, theirs);
    expect(output.text).toBe(['<antes>', ...ours, ...theirs, '<despues>'].join('\n'));
    // El `}` repetido se avisa; la linea en blanco no, que seria ruido.
    expect(output.duplicates).toBe(1);
  });

  it('tomar los dos bloques no cambia el significado de un YAML', () => {
    const ours = ['  a:', '    enabled: true', '    port: 1'];
    const theirs = ['  b:', '    enabled: true', '    port: 2'];
    const output = bothBlocks(ours, theirs);
    expect(output.text).toBe(['<antes>', ...ours, ...theirs, '<despues>'].join('\n'));
    expect(output.duplicates).toBe(1);
  });

  it('un bloque resuelto a vacio no aporta lineas', () => {
    const resolution = resolveEmpty(initResolution(DELETED_FILE), 0);
    const output = buildOutput(DELETED_FILE, resolution, LABELS);
    expect(output.text).toBe('// cabecera');
    expect(output.unresolved).toBe(0);
  });

  it('con edicion manual ensena el texto manual numerado', () => {
    const resolution = { ...initResolution(I18N_FILE), manual: 'a\nb' };
    const output = buildOutput(I18N_FILE, resolution, LABELS);
    expect(output.lines).toEqual([
      { kind: 'manual', text: 'a', number: 1 },
      { kind: 'manual', text: 'b', number: 2 },
    ]);
    expect(output.text).toBe('a\nb');
  });
});

describe('hunkMarkerText', () => {
  it('incluye el ancestro cuando el bloque lo trae y omite etiquetas vacias', () => {
    const text = hunkMarkerText(
      { kind: 'hunk', ours: ['a'], theirs: ['b'], base: ['o'] },
      { ours: '', theirs: 'otra' },
    );
    expect(text).toEqual(['<<<<<<<', 'a', '|||||||', 'o', '=======', 'b', '>>>>>>> otra']);
  });
});
