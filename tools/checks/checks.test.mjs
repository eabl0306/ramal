import assert from 'node:assert/strict';
import { after, describe, it } from 'node:test';

import { removeSandboxes, runCheck, sandbox } from './lib/sandbox.mjs';

// Los ejemplos se arman por concatenacion para que este fichero no viole las
// mismas reglas que comprueba.
const FOCUS = '.on' + 'ly(';
const SHORTHAND = 'f' + 'it(';
const MARKER = 'TO' + 'DO';
const ODD = 'XX' + 'X';

after(removeSandboxes);

describe('file-size', () => {
  it('acepta un fichero dentro del limite', () => {
    const root = sandbox({ 'packages/a/src/small.ts': 'const a = 1;\n'.repeat(10) });
    assert.equal(runCheck('file-size.mjs', root).code, 0);
  });

  it('rechaza un fichero de codigo de mas de 300 lineas', () => {
    const root = sandbox({ 'packages/a/src/big.ts': 'const a = 1;\n'.repeat(301) });
    const { code, output } = runCheck('file-size.mjs', root);
    assert.equal(code, 1);
    assert.match(output, /packages\/a\/src\/big\.ts/u);
  });

  it('revisa Rust en crates y en apps, y los ficheros sueltos de la raiz', () => {
    const root = sandbox({
      'crates/git/src/lib.rs': 'fn a() {}\n'.repeat(301),
      'apps/vscode-engine/src/main.rs': 'fn a() {}\n'.repeat(301),
      'big.mjs': 'const a = 1;\n'.repeat(301),
    });
    const { code, output } = runCheck('file-size.mjs', root);
    assert.equal(code, 1);
    assert.match(output, /crates\/git\/src\/lib\.rs/u);
    assert.match(output, /apps\/vscode-engine\/src\/main\.rs/u);
    assert.match(output, /^ {2}big\.mjs/mu);
  });

  it('deja a los tests llegar a 400 lineas, tambien los de Rust', () => {
    const root = sandbox({
      'apps/web/src/a.test.tsx': 'const a = 1;\n'.repeat(399),
      'apps/web/src/b.test.tsx': 'const a = 1;\n'.repeat(401),
      'crates/git/tests/status.rs': 'fn a() {}\n'.repeat(399),
    });
    const { code, output } = runCheck('file-size.mjs', root);
    assert.equal(code, 1);
    assert.match(output, /b\.test\.tsx/u);
    assert.doesNotMatch(output, /a\.test\.tsx/u);
    assert.doesNotMatch(output, /status\.rs/u);
  });

  it('ignora lo generado, los artefactos y lo que no es codigo fuente', () => {
    const big = 'const a = 1;\n'.repeat(500);
    const root = sandbox({
      'packages/bridge/src/protocol/generated/Commit.ts': big,
      'packages/api-client/src/generated/schema.ts': big,
      'apps/desktop/src-tauri/gen/schemas/desktop.ts': big,
      'apps/vscode/out/extension.js': big,
      'apps/vscode/.vscode-test/x.js': big,
      'apps/web/.react-router/types/root.ts': big,
      'docs/design/handoff/prototype.js': big,
      'docs/manual.md': 'texto\n'.repeat(500),
    });
    assert.equal(runCheck('file-size.mjs', root).code, 0);
  });
});

describe('no-focused-tests', () => {
  it('rechaza un test enfocado en TypeScript', () => {
    const root = sandbox({ 'packages/ui/src/a.test.ts': `describe${FOCUS}'x', () => {});\n` });
    const { code, output } = runCheck('no-focused-tests.mjs', root);
    assert.equal(code, 1);
    assert.match(output, /a\.test\.ts:1/u);
  });

  it('rechaza fdescribe y fit', () => {
    const root = sandbox({ 'apps/web/e2e/a.spec.ts': `${SHORTHAND}'x', () => {});\n` });
    assert.equal(runCheck('no-focused-tests.mjs', root).code, 1);
  });

  it('rechaza un #[ignore] de Rust sin motivo y acepta el que lo lleva', () => {
    const root = sandbox({
      'crates/a/src/lib.rs': '#[ignore]\nfn a() {}\n',
      'crates/b/src/lib.rs': '#[ignore = "necesita un repositorio remoto"]\nfn b() {}\n',
    });
    const { code, output } = runCheck('no-focused-tests.mjs', root);
    assert.equal(code, 1);
    assert.match(output, /crates\/a\/src\/lib\.rs/u);
    assert.doesNotMatch(output, /crates\/b\/src\/lib\.rs/u);
  });

  it('rechaza un #[ignore] con el motivo vacio', () => {
    const root = sandbox({ 'crates/a/src/lib.rs': '#[ignore = ""]\nfn a() {}\n' });
    assert.equal(runCheck('no-focused-tests.mjs', root).code, 1);
  });

  it('acepta un salto justificado con un comentario', () => {
    const root = sandbox({
      'apps/web/src/a.test.ts':
        "// se salta hasta que exista el endpoint\nit.skip('x', () => {});\n",
    });
    assert.equal(runCheck('no-focused-tests.mjs', root).code, 0);
  });
});

describe('todo-without-ticket', () => {
  it('rechaza un marcador sin ticket', () => {
    const root = sandbox({ 'packages/ui/src/a.ts': `// ${MARKER}: arreglar esto\n` });
    const { code, output } = runCheck('todo-without-ticket.mjs', root);
    assert.equal(code, 1);
    assert.match(output, /packages\/ui\/src\/a\.ts:1/u);
  });

  it('acepta una issue de GitHub o una tarjeta RAM como ticket', () => {
    const root = sandbox({
      'packages/ui/src/a.ts': `// ${MARKER}(#12): arreglar esto\n`,
      'crates/git/src/lib.rs': `// ${ODD}(RAM-7): caso raro\n`,
    });
    assert.equal(runCheck('todo-without-ticket.mjs', root).code, 0);
  });

  it('rechaza un ticket de otro tablero', () => {
    const root = sandbox({ 'crates/git/src/lib.rs': `// ${MARKER}(GAS-12): arreglar esto\n` });
    assert.equal(runCheck('todo-without-ticket.mjs', root).code, 1);
  });

  it('no confunde la palabra castellana todo con un marcador', () => {
    const root = sandbox({ 'packages/ui/src/a.ts': '// esto lo hace todo de golpe\n' });
    assert.equal(runCheck('todo-without-ticket.mjs', root).code, 0);
  });
});
