import assert from 'node:assert/strict';
import { after, describe, it } from 'node:test';

import { removeSandboxes, runCheck, sandbox } from './lib/sandbox.mjs';
import { parseTomlLite, tomlStrings, tomlValue } from './lib/toml.mjs';

after(removeSandboxes);

const ROOT_CARGO = [
  '[workspace]',
  'members = [',
  '  "crates/a", # el primero',
  '  "apps/*",',
  ']',
  '',
  '[workspace.lints.clippy]',
  'all = { level = "deny", priority = -1 }',
  '',
].join('\n');
const ROOT_CLIPPY = 'cognitive-complexity-threshold = 15\ntoo-many-lines-threshold = 50\n';
const WITH_LINTS = '[package]\nname = "a"\n\n[lints]\nworkspace = true\n';

function cargoRepo(extra) {
  return sandbox({
    'Cargo.toml': ROOT_CARGO,
    'clippy.toml': ROOT_CLIPPY,
    'crates/a/Cargo.toml': WITH_LINTS,
    'apps/b/Cargo.toml': WITH_LINTS,
    ...extra,
  });
}

describe('cargo-lints', () => {
  it('acepta un workspace en el que todos heredan los lints', () => {
    const { code, output } = runCheck('cargo-lints.mjs', cargoRepo({}));
    assert.equal(code, 0, output);
  });

  it('rechaza un miembro sin [lints] workspace = true, tambien si llega por un glob', () => {
    const root = cargoRepo({
      'apps/b/Cargo.toml': '[package]\nname = "b"\n\n[lints.clippy]\nall = "allow"\n',
      'crates/a/Cargo.toml': '[package]\nname = "a"\n# [lints]\n# workspace = true\n',
    });
    const { code, output } = runCheck('cargo-lints.mjs', root);
    assert.equal(code, 1);
    assert.match(output, /apps\/b\/Cargo\.toml\s+falta "\[lints\] workspace = true"/u);
    assert.match(output, /crates\/a\/Cargo\.toml\s+falta/u);
  });

  it('acepta las formas en linea de la misma clave', () => {
    const root = cargoRepo({
      'crates/a/Cargo.toml': 'lints.workspace = true\n[package]\nname = "a"\n',
      'apps/b/Cargo.toml': 'lints = { workspace = true }\n[package]\nname = "b"\n',
    });
    assert.equal(runCheck('cargo-lints.mjs', root).code, 0);
  });

  it('rechaza un clippy.toml anidado que no repite o cambia un umbral', () => {
    const root = cargoRepo({
      'crates/a/clippy.toml': 'cognitive-complexity-threshold = 15\n',
      'apps/b/clippy.toml': 'cognitive-complexity-threshold = 30\ntoo-many-lines-threshold = 50\n',
    });
    const { code, output } = runCheck('cargo-lints.mjs', root);
    assert.equal(code, 1);
    assert.match(output, /crates\/a\/clippy\.toml\s+too-many-lines-threshold.*no lo repite/u);
    assert.match(output, /apps\/b\/clippy\.toml\s+cognitive-complexity-threshold.*pone 30/u);
  });

  it('rechaza que la configuracion de cargo redirija clippy o apague lints', () => {
    const root = cargoRepo({
      '.cargo/config.toml': '[build]\nrustflags = ["-A", "clippy::too_many_lines"]\n',
    });
    const { code, output } = runCheck('cargo-lints.mjs', root);
    assert.equal(code, 1);
    assert.match(output, /\.cargo\/config\.toml/u);
  });
});

function tsRepo(extra) {
  return sandbox({
    'pnpm-workspace.yaml': 'packages:\n  - "apps/web"\n  - "packages/*"\n',
    'apps/web/package.json': { name: '@git-rudder/web' },
    'apps/web/eslint.config.js': 'export default [];\n',
    'apps/web/src/main.tsx': 'export const a = 1;\n',
    'packages/ui/package.json': { name: '@git-rudder/ui' },
    'packages/ui/eslint.config.js': 'export default [];\n',
    'packages/ui/src/index.ts': 'export const a = 1;\n',
    ...extra,
  });
}

describe('eslint-configs', () => {
  it('acepta paquetes con TypeScript que traen su configuracion', () => {
    const { code, output } = runCheck('eslint-configs.mjs', tsRepo({}));
    assert.equal(code, 0, output);
  });

  it('rechaza un paquete con TypeScript sin eslint.config.js propio', () => {
    const root = tsRepo({
      'packages/new/package.json': { name: '@git-rudder/new' },
      'packages/new/src/deep/index.tsx': 'export const a = 1;\n',
    });
    const { code, output } = runCheck('eslint-configs.mjs', root);
    assert.equal(code, 1);
    assert.match(output, /packages\/new\s+tiene TypeScript/u);
  });

  it('rechaza TypeScript fuera de cualquier paquete del workspace', () => {
    const root = tsRepo({ 'tools/scripts/release.ts': 'export const a = 1;\n' });
    const { code, output } = runCheck('eslint-configs.mjs', root);
    assert.equal(code, 1);
    assert.match(output, /tools\/scripts\/release\.ts\s+TypeScript fuera/u);
  });

  it('no pide configuracion por declaraciones, codigo generado ni JavaScript', () => {
    const root = tsRepo({
      'packages/config/package.json': { name: '@git-rudder/config' },
      'packages/config/index.js': 'export default {};\n',
      'packages/config/types.d.ts': 'export {};\n',
      'packages/gen/package.json': { name: '@git-rudder/gen' },
      'packages/gen/src/generated/schema.ts': 'export {};\n',
    });
    assert.equal(runCheck('eslint-configs.mjs', root).code, 0);
  });
});

describe('parseTomlLite', () => {
  it('lee tablas, arrays de varias lineas y comentarios fuera de las cadenas', () => {
    const entries = parseTomlLite(
      [
        'name = "a#b" # comentario',
        '[workspace]',
        'members = [',
        '  "x", # uno',
        '  "y",',
        ']',
      ].join('\n'),
    );
    assert.equal(tomlValue(entries, '', 'name'), '"a#b"');
    assert.deepEqual(tomlStrings(tomlValue(entries, 'workspace', 'members')), ['x', 'y']);
  });
});
