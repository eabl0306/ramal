import { spawnSync } from 'node:child_process';
import { mkdtempSync, mkdirSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const checksDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const sandboxes = [];

/**
 * Monta un repositorio de mentira en un directorio temporal. Los tests de los
 * scripts lo usan para probarlos de verdad: ficheros que violan la regla, el
 * script ejecutado como en el hook y el codigo de salida comprobado.
 */
export function sandbox(files) {
  const root = mkdtempSync(path.join(tmpdir(), 'git-rudder-checks-'));
  sandboxes.push(root);
  for (const [relativePath, content] of Object.entries(files)) {
    const absolute = path.join(root, relativePath);
    mkdirSync(path.dirname(absolute), { recursive: true });
    const text = typeof content === 'string' ? content : `${JSON.stringify(content, null, 2)}\n`;
    writeFileSync(absolute, text, 'utf8');
  }
  return root;
}

export function removeSandboxes() {
  for (const root of sandboxes.splice(0)) rmSync(root, { recursive: true, force: true });
}

export function runCheck(script, root) {
  const result = spawnSync(process.execPath, [path.join(checksDir, script), '--root', root], {
    encoding: 'utf8',
  });
  return { code: result.status, output: `${result.stdout}${result.stderr}` };
}
