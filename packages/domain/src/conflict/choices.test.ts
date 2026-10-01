import { describe, expect, it } from 'vitest';

import { DELETED_FILE, I18N_FILE, SESSION_FILE } from '../test/conflict-fixtures';

import { clearHunk, resolveEmpty, takeAll, toggleBlock, toggleLine } from './choices';
import { type FileResolution, hunksOf, initResolution } from './model';

function withManual(resolution: FileResolution): FileResolution {
  return { ...resolution, manual: 'texto', draft: 'borrador', acceptMarkers: true };
}

describe('initResolution', () => {
  it('empieza con todos los bloques sin decidir y ninguna linea elegida', () => {
    const resolution = initResolution(SESSION_FILE);
    expect(resolution.hunks).toHaveLength(2);
    expect(resolution.hunks[0]).toEqual({
      decision: 'none',
      ours: [false, false],
      theirs: [false, false, false, false],
    });
    expect(resolution.manual).toBeNull();
    expect(hunksOf(SESSION_FILE)).toHaveLength(2);
  });
});

describe('toggleLine', () => {
  it('marca una linea y el bloque pasa a decidido por lineas', () => {
    const next = toggleLine(initResolution(I18N_FILE), 0, 'ours', 0);
    expect(next.hunks[0]).toEqual({ decision: 'lines', ours: [true], theirs: [false, false] });
  });

  it('desmarcar la ultima linea devuelve el bloque a sin decidir', () => {
    const once = toggleLine(initResolution(I18N_FILE), 0, 'theirs', 1);
    const twice = toggleLine(once, 0, 'theirs', 1);
    expect(twice.hunks[0]?.decision).toBe('none');
  });

  it('ignora bloques y lineas fuera de rango', () => {
    const initial = initResolution(I18N_FILE);
    expect(toggleLine(initial, 5, 'ours', 0)).toBe(initial);
    expect(toggleLine(initial, 0, 'ours', 9).hunks[0]).toEqual(initial.hunks[0]);
  });

  it('descarta la edicion manual porque se escribio sobre otras elecciones', () => {
    const next = toggleLine(withManual(initResolution(I18N_FILE)), 0, 'ours', 0);
    expect(next.manual).toBeNull();
    expect(next.draft).toBeNull();
    expect(next.acceptMarkers).toBe(false);
  });
});

describe('toggleBlock', () => {
  it('toma el bloque entero de un lado sin tocar el otro', () => {
    const start = toggleLine(initResolution(SESSION_FILE), 0, 'ours', 0);
    const next = toggleBlock(SESSION_FILE, start, 0, 'theirs');
    expect(next.hunks[0]).toEqual({
      decision: 'lines',
      ours: [true, false],
      theirs: [true, true, true, true],
    });
  });

  it('quita el bloque si ya estaba entero', () => {
    const taken = toggleBlock(SESSION_FILE, initResolution(SESSION_FILE), 1, 'ours');
    const dropped = toggleBlock(SESSION_FILE, taken, 1, 'ours');
    expect(dropped.hunks[1]).toEqual({
      decision: 'none',
      ours: [false, false],
      theirs: [false, false],
    });
  });

  it('con un lado vacio resuelve el bloque a vacio y al repetir lo deshace', () => {
    const initial = toggleLine(initResolution(DELETED_FILE), 0, 'ours', 0);
    const empty = toggleBlock(DELETED_FILE, initial, 0, 'theirs');
    expect(empty.hunks[0]).toEqual({ decision: 'empty', ours: [false], theirs: [] });
    const undone = toggleBlock(DELETED_FILE, empty, 0, 'theirs');
    expect(undone.hunks[0]?.decision).toBe('none');
  });

  it('ignora un bloque que no existe', () => {
    const initial = initResolution(DELETED_FILE);
    expect(toggleBlock(DELETED_FILE, initial, 3, 'ours')).toBe(initial);
  });
});

describe('resolveEmpty y clearHunk', () => {
  it('resolveEmpty limpia las lineas y deja el bloque resuelto a nada', () => {
    const picked = toggleLine(initResolution(I18N_FILE), 0, 'ours', 0);
    expect(resolveEmpty(picked, 0).hunks[0]).toEqual({
      decision: 'empty',
      ours: [false],
      theirs: [false, false],
    });
  });

  it('clearHunk devuelve el bloque a sin resolver', () => {
    const empty = resolveEmpty(initResolution(I18N_FILE), 0);
    expect(clearHunk(empty, 0).hunks[0]?.decision).toBe('none');
  });
});

describe('takeAll', () => {
  it('toma un lado entero en cada bloque y suelta el otro', () => {
    const start = toggleLine(initResolution(SESSION_FILE), 1, 'ours', 1);
    const next = takeAll(SESSION_FILE, start, 'theirs');
    expect(next.hunks).toEqual([
      { decision: 'lines', ours: [false, false], theirs: [true, true, true, true] },
      { decision: 'lines', ours: [false, false], theirs: [true, true] },
    ]);
  });

  it('un bloque con ese lado vacio queda resuelto a vacio', () => {
    const next = takeAll(DELETED_FILE, initResolution(DELETED_FILE), 'theirs');
    expect(next.hunks[0]?.decision).toBe('empty');
  });

  it('descarta la edicion manual', () => {
    const next = takeAll(I18N_FILE, withManual(initResolution(I18N_FILE)), 'ours');
    expect(next.manual).toBeNull();
  });
});
