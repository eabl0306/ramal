import assert from 'node:assert/strict';
import { after, describe, it } from 'node:test';

import { removeSandboxes, runCheck, sandbox } from './lib/sandbox.mjs';

const SCRIPT = 'rust-test-modules.mjs';

after(removeSandboxes);

describe('rust-test-modules', () => {
  it('rechaza un modulo de test en linea en el codigo de produccion', () => {
    const root = sandbox({
      'apps/api/crates/domain/src/latency.rs':
        'pub fn a() {}\n\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn b() {}\n}\n',
      'crates/git/src/lib.rs':
        '#[cfg(all(test, unix))]\n#[allow(dead_code, reason = "x")]\npub(crate) mod checks { }\n',
    });
    const { code, output } = runCheck(SCRIPT, root);
    assert.equal(code, 1);
    assert.match(output, /latency\.rs:3\s+mod tests en linea/u);
    assert.match(output, /lib\.rs:1\s+mod checks en linea/u);
  });

  it('acepta el modulo en su fichero, las ayudas de testing/ y los tests de tests/', () => {
    const root = sandbox({
      'crates/core-application/src/use_case.rs': 'pub fn a() {}\n\n#[cfg(test)]\nmod tests;\n',
      'crates/core-application/src/use_case/tests.rs': 'mod helpers {\n    pub fn h() {}\n}\n',
      'crates/core-application/src/testing/fake.rs': '#[cfg(test)]\nmod tests {\n}\n',
      'crates/git/tests/status.rs': '#[cfg(test)]\nmod inner {\n}\n',
      'crates/git/src/doc.rs':
        '// #[cfg(test)]\n// mod tests {\nconst S: &str = "#[cfg(test)] mod t {";\n',
    });
    const { code, output } = runCheck(SCRIPT, root);
    assert.equal(code, 0, output);
  });
});
