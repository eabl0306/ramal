import { describe, expect, it } from 'vitest';

import { isValidRefName, refNameIssues, suggestBranchName, validatePromptValue } from './ref-name';

describe('refNameIssues', () => {
  it('acepta nombres normales con barras', () => {
    expect(isValidRefName('feature/graph-ui')).toBe(true);
    expect(isValidRefName('v2.4.0')).toBe(true);
  });

  it('rechaza cada regla de git check-ref-format', () => {
    const cases: [string, string][] = [
      ['', 'empty'],
      ['con espacio', 'forbidden-character'],
      ['a~b', 'forbidden-character'],
      ['a\u0007b', 'forbidden-character'],
      ['a\u007fb', 'forbidden-character'],
      ['a..b', 'double-dot'],
      ['-rama', 'leading-dash'],
      ['/rama', 'slash-position'],
      ['rama/', 'slash-position'],
      ['a//b', 'slash-position'],
      ['rama.', 'trailing-dot'],
      ['a/.oculta', 'dot-component'],
      ['rama.lock', 'lock-suffix'],
      ['a@{b', 'at-brace'],
      ['@', 'only-at'],
    ];
    for (const [name, issue] of cases) expect(refNameIssues(name)).toContain(issue);
  });

  it('devuelve todos los problemas a la vez', () => {
    expect(refNameIssues('-a..b.')).toEqual(['double-dot', 'leading-dash', 'trailing-dot']);
  });
});

describe('suggestBranchName', () => {
  it('recorta y cambia los espacios por guiones', () => {
    expect(suggestBranchName('  nueva   rama de prueba ')).toBe('nueva-rama-de-prueba');
  });
});

describe('validatePromptValue', () => {
  it('aplica la regla declarativa de cada peticion de texto', () => {
    expect(validatePromptValue('branch-name', ' feature/x ')).toEqual([]);
    expect(validatePromptValue('tag-name', 'v1..2')).toEqual(['double-dot']);
    expect(validatePromptValue('non-empty', '  ')).toEqual(['empty']);
    expect(validatePromptValue('non-empty', 'algo')).toEqual([]);
    expect(validatePromptValue('none', '')).toEqual([]);
  });
});
