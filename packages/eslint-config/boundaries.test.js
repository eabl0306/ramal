import assert from 'node:assert/strict';
import path from 'node:path';
import { describe, it } from 'node:test';
import { fileURLToPath } from 'node:url';

import { ESLint } from 'eslint';

import { boundariesConfig } from './boundaries.js';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..');
const minimal = { elements: [], policies: [] };

describe('boundariesConfig', () => {
  it('exige rootPath y que sea absoluto', () => {
    assert.throws(() => boundariesConfig(minimal), /rootPath/u);
    assert.throws(() => boundariesConfig({ ...minimal, rootPath: 'apps/web' }), /rootPath/u);
  });

  it('pasa rootPath al plugin para que no dependa del directorio de trabajo', () => {
    const rootPath = path.join(repoRoot, 'apps', 'web');
    const [config] = boundariesConfig({ ...minimal, rootPath });
    assert.equal(config?.settings?.['boundaries/root-path'], rootPath);
  });

  // Existe por un fallo real: el hook de pre-commit lanza ESLint desde la raiz y
  // las reglas de capas no casaban con nada, asi que un import de React en el
  // dominio de la web entraba en el commit.
  it('aplica las capas aunque ESLint se lance desde la raiz del repo, como el hook', async () => {
    const eslint = new ESLint({ cwd: repoRoot });
    const filePath = path.join(repoRoot, 'apps', 'web', 'src', 'domain', 'downloads', 'release.ts');
    const code = "import { useState } from 'react';\n\nexport const probe = useState;\n";
    const [result] = await eslint.lintText(code, { filePath });
    const rules = (result?.messages ?? []).map((message) => message.ruleId);
    assert.ok(rules.includes('boundaries/dependencies'), JSON.stringify(result?.messages));
  });
});
