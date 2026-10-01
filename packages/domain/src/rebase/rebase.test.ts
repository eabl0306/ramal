import { describe, expect, it } from 'vitest';

import {
  createRebasePlan,
  moveRebaseItem,
  setRebaseAction,
  setRebaseMessage,
  summarizeRebasePlan,
} from './plan';
import { rebaseTodo } from './todo';
import { canStartRebase, validateRebasePlan } from './validate';

/** Los tres commits del ejemplo del analisis, el mas antiguo primero. */
const COMMITS = [
  { sha: '9d04c6a', subject: 'Estructura base del componente Graph' },
  { sha: '3c9d8e4', subject: 'Dibujar carriles con curvas bézier' },
  { sha: 'a41f9c2', subject: 'Añadir zoom y desplazamiento al grafo' },
];

const plan = createRebasePlan('develop', COMMITS);

describe('plan de rebase', () => {
  it('empieza con todo en pick y el mensaje igual al asunto', () => {
    expect(plan.onto).toBe('develop');
    expect(plan.items.map((item) => item.action)).toEqual(['pick', 'pick', 'pick']);
    expect(plan.items[2]?.message).toBe('Añadir zoom y desplazamiento al grafo');
  });

  it('cambia la accion y el mensaje de un commit sin tocar los demas', () => {
    const next = setRebaseMessage(setRebaseAction(plan, 2, 'reword'), 2, 'feat(graph): zoom');
    expect(next.items[2]).toMatchObject({ action: 'reword', message: 'feat(graph): zoom' });
    expect(next.items[0]).toBe(plan.items[0]);
    expect(setRebaseAction(plan, 9, 'drop')).toBe(plan);
  });

  it('mueve commits y no hace nada fuera de rango', () => {
    const moved = moveRebaseItem(plan, 2, 0);
    expect(moved.items.map((item) => item.sha)).toEqual(['a41f9c2', '9d04c6a', '3c9d8e4']);
    expect(moveRebaseItem(plan, 0, -1)).toBe(plan);
    expect(moveRebaseItem(plan, 0, 3)).toBe(plan);
    expect(moveRebaseItem(plan, 1, 1)).toBe(plan);
  });

  it('resume reword y edit como pick y fixup como squash', () => {
    let next = setRebaseAction(plan, 1, 'fixup');
    next = setRebaseAction(next, 2, 'reword');
    expect(summarizeRebasePlan(next)).toEqual({ pick: 2, squash: 1, drop: 0 });
    expect(summarizeRebasePlan(setRebaseAction(next, 0, 'drop'))).toEqual({
      pick: 1,
      squash: 1,
      drop: 1,
    });
  });
});

describe('validateRebasePlan', () => {
  it('un plan normal no tiene problemas', () => {
    const issues = validateRebasePlan(plan);
    expect(issues).toEqual([]);
    expect(canStartRebase(issues)).toBe(true);
  });

  it('un plan vacio es un error', () => {
    expect(validateRebasePlan(createRebasePlan('main', []))).toEqual([
      { code: 'empty', severity: 'error' },
    ]);
  });

  it('el primer commit no puede ser squash ni fixup, aunque haya drops delante', () => {
    const fixupFirst = moveRebaseItem(setRebaseAction(plan, 1, 'fixup'), 1, 0);
    expect(validateRebasePlan(fixupFirst)).toEqual([
      { code: 'needs-previous-commit', severity: 'error', index: 0, action: 'fixup' },
    ]);
    const behindDrop = setRebaseAction(setRebaseAction(plan, 0, 'drop'), 1, 'squash');
    const issues = validateRebasePlan(behindDrop);
    expect(issues).toEqual([
      { code: 'needs-previous-commit', severity: 'error', index: 1, action: 'squash' },
    ]);
    expect(canStartRebase(issues)).toBe(false);
  });

  it('reword necesita un mensaje que no este vacio', () => {
    const next = setRebaseMessage(setRebaseAction(plan, 1, 'reword'), 1, '   ');
    expect(validateRebasePlan(next)).toEqual([
      { code: 'reword-without-message', severity: 'error', index: 1 },
    ]);
  });

  it('avisa si se descarta todo y si hara falta force push, sin bloquear', () => {
    let allDropped = plan;
    for (const index of [0, 1, 2]) allDropped = setRebaseAction(allDropped, index, 'drop');
    const issues = validateRebasePlan(allDropped, { pushed: true });
    expect(issues).toEqual([
      { code: 'all-dropped', severity: 'warning' },
      { code: 'needs-force-push', severity: 'warning' },
    ]);
    expect(canStartRebase(issues)).toBe(true);
  });

  it('un arbol sucio bloquea salvo con autostash', () => {
    expect(validateRebasePlan(plan, { dirtyTree: true })).toEqual([
      { code: 'dirty-tree', severity: 'error' },
    ]);
    expect(validateRebasePlan(plan, { dirtyTree: true, autostash: true })).toEqual([]);
  });
});

describe('rebaseTodo', () => {
  it('escribe el todo del ejemplo, con la primera linea arriba', () => {
    let next = setRebaseAction(plan, 1, 'fixup');
    next = setRebaseAction(next, 2, 'reword');
    expect(rebaseTodo(next)).toBe(
      [
        'pick 9d04c6a Estructura base del componente Graph',
        'fixup 3c9d8e4 Dibujar carriles con curvas bézier',
        'reword a41f9c2 Añadir zoom y desplazamiento al grafo',
        '',
      ].join('\n'),
    );
  });

  it('escribe los drop explicitos y usa solo la primera linea del asunto', () => {
    const multiline = createRebasePlan('main', [{ sha: 'abc', subject: 'Asunto\n\ncuerpo' }]);
    expect(rebaseTodo(setRebaseAction(multiline, 0, 'drop'))).toBe('drop abc Asunto\n');
    expect(rebaseTodo(createRebasePlan('main', []))).toBe('');
  });
});
