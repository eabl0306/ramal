import { describe, expect, it } from 'vitest';

import { checkConventional, formatConventional, parseConventional } from './conventional';
import {
  canCommit,
  composeCommitMessage,
  countGraphemes,
  splitCommitMessage,
  summaryCounter,
} from './message';
import { cherryPickMessage, mergeMessage, revertMessage } from './templates';

describe('contador del resumen', () => {
  it('cuenta grafemas, no unidades UTF-16', () => {
    expect('👩‍💻'.length).toBeGreaterThan(1);
    expect(countGraphemes('👩‍💻')).toBe(1);
    expect(countGraphemes('café')).toBe(4);
  });

  it('avisa al pasar de 72 sin cortar', () => {
    expect(summaryCounter('a'.repeat(72))).toEqual({ length: 72, remaining: 0, over: false });
    expect(summaryCounter('a'.repeat(73))).toEqual({ length: 73, remaining: -1, over: true });
    expect(summaryCounter('abc', 2).over).toBe(true);
  });
});

describe('canCommit', () => {
  it('necesita resumen y ficheros preparados, salvo con amend', () => {
    expect(canCommit({ summary: 'feat: x', stagedCount: 1, amend: false })).toBe(true);
    expect(canCommit({ summary: 'feat: x', stagedCount: 0, amend: false })).toBe(false);
    expect(canCommit({ summary: 'feat: x', stagedCount: 0, amend: true })).toBe(true);
    expect(canCommit({ summary: '   ', stagedCount: 3, amend: false })).toBe(false);
  });
});

describe('mensaje completo', () => {
  it('parte resumen y cuerpo sin perder el cuerpo', () => {
    expect(splitCommitMessage('feat: zoom\r\n\r\n- linea 1\n- linea 2\n')).toEqual({
      summary: 'feat: zoom',
      body: '- linea 1\n- linea 2',
    });
    expect(splitCommitMessage('  solo resumen ')).toEqual({ summary: 'solo resumen', body: '' });
  });

  it('compone con una linea en blanco entre resumen y cuerpo', () => {
    expect(composeCommitMessage({ summary: 'fix: x ', body: ' detalle ' })).toBe(
      'fix: x\n\ndetalle',
    );
    expect(composeCommitMessage({ summary: 'fix: x', body: '  ' })).toBe('fix: x');
  });
});

describe('Conventional Commits', () => {
  it('parsea tipo, ambito, ruptura y asunto', () => {
    expect(parseConventional('feat(graph)!: zoom y desplazamiento')).toEqual({
      type: 'feat',
      scope: 'graph',
      breaking: true,
      subject: 'zoom y desplazamiento',
    });
    expect(parseConventional('fix: login')).toMatchObject({ scope: null, breaking: false });
    expect(parseConventional('Añadir zoom')).toBeNull();
  });

  it('comprueba formato, tipo conocido y asunto', () => {
    expect(checkConventional('feat(ui): botones')).toMatchObject({ ok: true });
    expect(checkConventional('zoom al grafo')).toEqual({ ok: false, reason: 'format' });
    expect(checkConventional('wip: algo')).toEqual({ ok: false, reason: 'unknown-type' });
    expect(checkConventional('feat:  ')).toEqual({ ok: false, reason: 'empty-subject' });
    expect(checkConventional('wip: algo', ['wip'])).toMatchObject({ ok: true });
  });

  it('formatea de vuelta', () => {
    expect(
      formatConventional({ type: 'feat', scope: 'graph', breaking: false, subject: 'zoom' }),
    ).toBe('feat(graph): zoom');
    expect(formatConventional({ type: 'fix', scope: '', breaking: true, subject: ' x ' })).toBe(
      'fix!: x',
    );
  });
});

describe('mensajes de git', () => {
  it('merge omite la rama destino en main y master', () => {
    expect(mergeMessage('hotfix/login', 'main')).toBe("Merge branch 'hotfix/login'");
    expect(mergeMessage('hotfix/login', 'develop')).toBe(
      "Merge branch 'hotfix/login' into develop",
    );
  });

  it('revert y cherry-pick siguen el formato de git', () => {
    expect(revertMessage('Añadir zoom', 'a41f9c2')).toBe(
      'Revert "Añadir zoom"\n\nThis reverts commit a41f9c2.',
    );
    expect(cherryPickMessage('Añadir zoom\n', 'a41f9c2e')).toBe(
      'Añadir zoom\n\n(cherry picked from commit a41f9c2e)',
    );
  });
});
