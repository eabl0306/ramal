// Repositorio de prueba para los tests de integracion: tres commits en main y
// una rama, creados con el git de la maquina en una carpeta temporal. Las
// fechas y la identidad van fijas para que el grafo sea siempre el mismo.
import { execFileSync } from 'node:child_process';
import { mkdtempSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';

/** Resumenes de los commits, del mas nuevo al mas viejo, como los pinta el grafo. */
export const FIXTURE_SUMMARIES = ['Tercer commit', 'Segundo commit', 'Primer commit'];

const IDENTITY = {
  GIT_AUTHOR_NAME: 'Ramal Test',
  GIT_AUTHOR_EMAIL: 'test@ramal.invalid',
  GIT_COMMITTER_NAME: 'Ramal Test',
  GIT_COMMITTER_EMAIL: 'test@ramal.invalid',
};

function git(cwd, args, date) {
  execFileSync('git', ['-c', 'commit.gpgsign=false', '-c', 'init.defaultBranch=main', ...args], {
    cwd,
    stdio: 'ignore',
    env: { ...process.env, ...IDENTITY, GIT_AUTHOR_DATE: date, GIT_COMMITTER_DATE: date },
  });
}

/** Crea el repositorio y devuelve su ruta. */
export function createFixtureRepo() {
  const root = mkdtempSync(path.join(tmpdir(), 'ramal-vscode-it-'));
  git(root, ['init', '--quiet'], '2026-01-01T10:00:00+00:00');
  const summaries = [...FIXTURE_SUMMARIES].reverse();
  summaries.forEach((summary, index) => {
    const date = `2026-01-0${String(index + 1)}T10:00:00+00:00`;
    writeFileSync(path.join(root, `file-${String(index)}.txt`), `${summary}\n`);
    git(root, ['add', '--all'], date);
    git(root, ['commit', '--quiet', '-m', summary], date);
    if (index === 1) git(root, ['branch', 'feature/graph'], date);
  });
  return root;
}
