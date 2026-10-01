import assert from 'node:assert/strict';
import { after, describe, it } from 'node:test';

import { removeSandboxes, runCheck, sandbox } from './lib/sandbox.mjs';

after(removeSandboxes);

const ROOT_CARGO = [
  '[workspace]',
  'members = [',
  '  "crates/*",',
  '  "apps/api/crates/api",',
  '  "tools/complexity",',
  ']',
  '',
].join('\n');
const CRATE = '[package]\nname = "x"\n\n[lints]\nworkspace = true\n';
const CARGO_DOCKERFILE = 'FROM rust\nRUN cargo build --release\n';

function repo(ignore, extra = {}) {
  return sandbox({
    'Cargo.toml': ROOT_CARGO,
    'crates/a/Cargo.toml': CRATE,
    'apps/api/crates/api/Cargo.toml': CRATE,
    'tools/complexity/Cargo.toml': CRATE,
    'apps/api/Dockerfile': CARGO_DOCKERFILE,
    'apps/api/Dockerfile.dockerignore': ignore,
    ...extra,
  });
}

describe('docker-context', () => {
  it('acepta una lista blanca que readmite a todos los miembros', () => {
    const ignore =
      '# lista blanca\n*\n\n!Cargo.toml\n!crates\n!apps/api/crates/\n!tools/complexity\n';
    const { code, output } = runCheck('docker-context.mjs', repo(ignore));
    assert.equal(code, 0, output);
  });

  it('rechaza un miembro que la lista blanca no readmite', () => {
    const ignore = '*\n!Cargo.toml\n!crates\n!apps/api/crates\n';
    const { code, output } = runCheck('docker-context.mjs', repo(ignore));
    assert.equal(code, 1);
    assert.match(output, /apps\/api\/Dockerfile\.dockerignore\s+falta "!tools\/complexity"/u);
  });

  it('no confunde un prefijo de nombre con un directorio que lo contiene', () => {
    const ignore = '*\n!crates\n!apps/api/crates\n!tools/comp\n';
    assert.equal(runCheck('docker-context.mjs', repo(ignore)).code, 1);
  });

  it('ignora los contextos que no compilan Rust o que no son lista blanca', () => {
    const extra = {
      'apps/web/Dockerfile': 'FROM node\nRUN corepack pnpm build\n',
      'apps/web/Dockerfile.dockerignore': '*\n!apps/web\n',
    };
    const ignore = '**/target\n**/node_modules\n';
    assert.equal(runCheck('docker-context.mjs', repo(ignore, extra)).code, 0);
  });
});
