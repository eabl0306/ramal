import { describe, expect, it } from 'vitest';

import { I18N_FILE, LABELS, SESSION_FILE } from '../test/conflict-fixtures';

import { buildOutput } from './build-output';
import { findConflictMarkers, hasConflictMarkers, parseConflictMarkers } from './markers';
import { initResolution } from './model';

describe('findConflictMarkers', () => {
  it('detecta los cuatro marcadores, incluido el del ancestro', () => {
    const text = ['<<<<<<< HEAD', 'a', '||||||| base', 'o', '=======', 'b', '>>>>>>> rama'].join(
      '\n',
    );
    expect(findConflictMarkers(text).map((marker) => marker.kind)).toEqual([
      'start',
      'base',
      'separator',
      'end',
    ]);
  });

  it('un subrayado de Markdown fuera de un bloque no es un marcador', () => {
    expect(hasConflictMarkers('Titulo\n=======\ntexto')).toBe(false);
  });

  it('un marcador que no empieza la linea no cuenta', () => {
    expect(hasConflictMarkers('a = "=======";')).toBe(false);
    expect(hasConflictMarkers('a\n<<<<<<< x\nb')).toBe(true);
  });

  it('reconoce marcadores con fin de linea CRLF y da el numero de linea', () => {
    expect(findConflictMarkers('uno\r\n<<<<<<< HEAD\r\n')).toEqual([{ line: 2, kind: 'start' }]);
  });
});

describe('parseConflictMarkers', () => {
  it('es la inversa de buildOutput sin elecciones', () => {
    for (const file of [SESSION_FILE, I18N_FILE]) {
      const text = buildOutput(file, initResolution(file), LABELS).text;
      const parsed = parseConflictMarkers(text);
      expect(parsed).toEqual({ ok: true, segments: file.segments, labels: LABELS });
    }
  });

  it('guarda las lineas del ancestro y su etiqueta', () => {
    const parsed = parseConflictMarkers(
      ['x', '<<<<<<< HEAD', 'a', '||||||| base', 'o', '=======', 'b', '>>>>>>> rama'].join('\n'),
    );
    expect(parsed).toEqual({
      ok: true,
      segments: [
        { kind: 'context', lines: ['x'] },
        { kind: 'hunk', ours: ['a'], theirs: ['b'], base: ['o'] },
      ],
      labels: { ours: 'HEAD', theirs: 'rama', base: 'base' },
    });
  });

  it('trata como texto un separador fuera de un bloque', () => {
    const parsed = parseConflictMarkers('Titulo\n=======');
    expect(parsed).toEqual({
      ok: true,
      segments: [{ kind: 'context', lines: ['Titulo', '======='] }],
      labels: null,
    });
  });

  it('falla con un bloque sin cerrar', () => {
    expect(parseConflictMarkers('<<<<<<< a\nx\n=======\ny')).toEqual({
      ok: false,
      error: 'unterminated',
      line: 4,
    });
  });

  it('falla con un marcador fuera de sitio', () => {
    expect(parseConflictMarkers('<<<<<<< a\nx\n<<<<<<< b')).toEqual({
      ok: false,
      error: 'unexpected-marker',
      line: 3,
    });
    expect(parseConflictMarkers('>>>>>>> suelto')).toEqual({
      ok: false,
      error: 'unexpected-marker',
      line: 1,
    });
  });
});
