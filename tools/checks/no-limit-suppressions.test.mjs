import assert from 'node:assert/strict';
import { after, describe, it } from 'node:test';

import { blankRustNoise } from './lib/rust-source.mjs';
import { removeSandboxes, runCheck, sandbox } from './lib/sandbox.mjs';

// Las directivas se arman por concatenacion: el script busca por texto y este
// fichero tambien lo revisa.
const DISABLE = 'eslint-' + 'disable';
const NEXT_LINE = `${DISABLE}-next-line`;
const SCRIPT = 'no-limit-suppressions.mjs';

after(removeSandboxes);

function expectViolations(files, expected) {
  const { code, output } = runCheck(SCRIPT, sandbox(files));
  assert.equal(code, 1, output);
  for (const fragment of expected) assert.match(output, fragment);
  return output;
}

describe('no-limit-suppressions en TypeScript', () => {
  it('rechaza una directiva de bloque sin reglas, que lo apaga todo', () => {
    expectViolations({ 'packages/domain/src/a.ts': `/* ${DISABLE} */\nexport const a = 1;\n` }, [
      /a\.ts:1\s.*sin reglas/u,
    ]);
  });

  it('rechaza una directiva de linea sin reglas', () => {
    expectViolations({ 'apps/web/src/a.ts': `// ${NEXT_LINE}\nexport const a = 1;\n` }, [
      /a\.ts:1\s.*sin reglas/u,
    ]);
  });

  it('rechaza apagar un limite aunque lleve motivo', () => {
    const rules = [
      'complexity',
      'max-lines',
      'max-lines-per-function',
      'max-depth',
      'max-params',
      'max-nested-callbacks',
      'sonarjs/cognitive-complexity',
      'boundaries/dependencies',
    ];
    const lines = rules.map((rule) => `// ${NEXT_LINE} ${rule} -- es largo`);
    const output = expectViolations(
      { 'packages/ui/src/a.test.tsx': `${lines.join('\n')}\n` },
      rules.map((rule) => new RegExp(`limite apagado con ${rule.replace('/', '\\/')}:`, 'u')),
    );
    assert.equal(output.match(/limite apagado/gu)?.length, rules.length);
  });

  it('rechaza una directiva sin motivo y una de varias lineas que apaga un limite', () => {
    expectViolations(
      {
        'apps/desktop/src/a.ts': `// ${NEXT_LINE} no-console\nconsole.log(1);\n`,
        'apps/vscode/src/b.ts': `/* ${DISABLE}\n   complexity,\n   max-depth -- motivo */\n`,
      },
      [/a\.ts:1\s.*sin motivo/u, /b\.ts:1\s.*complexity/u, /b\.ts:1\s.*max-depth/u],
    );
  });

  it('rechaza la configuracion de reglas en linea', () => {
    const config = `/* ${'esl' + 'int'} complexity: ["error", 99] */\n`;
    expectViolations({ 'packages/ui/src/a.ts': config }, [/configuracion de reglas en linea/u]);
  });

  it('acepta una directiva con regla concreta y motivo', () => {
    const root = sandbox({
      'apps/desktop/src/a.ts': `// ${NEXT_LINE} @typescript-eslint/only-throw-error -- objeto plano de Tauri\n`,
      'packages/ui/src/b.ts': `/* ${DISABLE} no-console -- script de depuracion */\n`,
    });
    assert.equal(runCheck(SCRIPT, root).code, 0);
  });
});

describe('no-limit-suppressions en Rust', () => {
  it('rechaza un allow sobre un limite aunque lleve motivo', () => {
    const attribute = [
      '#[allow(',
      '    clippy::too_many_lines,',
      '    clippy::too_many_arguments,',
      '    reason = "es largo"',
      ')]',
      'fn a() {}',
      '#[expect(clippy::excessive_nesting, reason = "x")]',
      'fn b() {}',
      '#![allow(clippy::cognitive_complexity, reason = "x")]',
    ].join('\n');
    expectViolations({ 'crates/core-domain/src/lib.rs': `${attribute}\n` }, [
      /lib\.rs:1\s.*too_many_lines/u,
      /lib\.rs:1\s.*too_many_arguments/u,
      /lib\.rs:7\s.*excessive_nesting/u,
      /lib\.rs:9\s.*cognitive_complexity/u,
    ]);
  });

  it('rechaza apagar el grupo que contiene el limite y el alias antiguo', () => {
    expectViolations(
      {
        'crates/git/src/a.rs': '#[allow(clippy::pedantic, reason = "x")]\nfn a() {}\n',
        'crates/git/src/b.rs': '#[cfg_attr(test, allow(clippy::complexity))]\nfn b() {}\n',
        'crates/git/src/c.rs': '#[allow(clippy::cyclomatic_complexity, reason = "x")]\n',
      },
      [/a\.rs:1\s.*pedantic/u, /b\.rs:1\s.*complexity/u, /c\.rs:1\s.*cyclomatic_complexity/u],
    );
  });

  it('rechaza cambiar el umbral de clippy con un atributo', () => {
    expectViolations(
      { 'crates/git/src/a.rs': '#[clippy::cognitive_complexity = "99"]\nfn a() {}\n' },
      [/umbral de clippy cambiado en linea/u],
    );
  });

  it('deja apagar too_many_lines en un test, como TypeScript, pero no la complejidad', () => {
    const allow = (lint) => `#[allow(clippy::${lint}, reason = "fixture largo")]\nfn a() {}\n`;
    const output = expectViolations(
      {
        'crates/git/tests/long.rs': allow('too_many_lines'),
        'crates/git/tests/complex.rs': allow('cognitive_complexity'),
      },
      [/complex\.rs:1/u],
    );
    assert.doesNotMatch(output, /long\.rs/u);
  });

  it('no confunde un comentario ni el texto del motivo con un allow', () => {
    const root = sandbox({
      'crates/git/src/a.rs': [
        '// #[allow(clippy::too_many_lines)]',
        '/* #[allow(clippy::too_many_arguments)] */',
        '#[allow(clippy::unwrap_used, reason = "nada de clippy::too_many_lines aqui")]',
        'fn a() {}',
        'const S: &str = "#[allow(clippy::too_many_lines)]";',
        '',
      ].join('\n'),
    });
    assert.equal(runCheck(SCRIPT, root).code, 0);
  });
});

describe('blankRustNoise', () => {
  it('borra comentarios y literales y deja intactos los lifetimes y las lineas', () => {
    const source = [
      'fn a<\'x>(c: char) -> &\'x str { if c == \'\\\'\' { "a\\"b" } else { r#"c"d"# } }',
      '/* fuera /* anidado */ todavia fuera */ fn b() {}',
      'let q = \'"\'; let s = br"x\\"; // fin',
    ].join('\n');
    const clean = blankRustNoise(source);
    assert.equal(clean.split('\n').length, 3);
    assert.match(clean, /fn a<'x>\(c: char\) -> &'x str/u);
    assert.doesNotMatch(clean, /a\\"b|c"d|anidado|todavia|fin/u);
    assert.match(clean, /\s+fn b\(\) \{\}/u);
    assert.match(clean, /let s = br"\s*";/u);
  });
});
