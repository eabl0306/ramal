import { execFileSync } from 'node:child_process';
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';

/** Los commits del repositorio de prueba, del mas antiguo al mas nuevo. */
export const FIXTURE_COMMITS = ['Primer commit', 'Segundo commit', 'Tercer commit'] as const;

export interface FixtureRepo {
  /** Raiz del arbol de trabajo. */
  readonly path: string;
  readonly dispose: () => void;
}

/** Primer instante de los commits: 2026-09-30T10:00:00+02:00. */
const FIRST_COMMIT_SECONDS = 1_790_755_200;

/**
 * Un repositorio real con tres commits en un tempdir, creado con el git del
 * sistema. El entorno se fija como en los tests del motor (HOME propio, sin
 * configuracion de sistema, autor y fechas fijos) para que la configuracion de
 * quien corre el E2E no se cuele.
 */
export function createFixtureRepo(): FixtureRepo {
  const base = mkdtempSync(path.join(tmpdir(), 'ramal-e2e-'));
  const home = path.join(base, 'home');
  const repo = path.join(base, 'repo');
  mkdirSync(home);
  mkdirSync(repo);
  writeFileSync(path.join(home, '.gitconfig'), '');
  const git = (args: readonly string[], seconds = FIRST_COMMIT_SECONDS): void => {
    execFileSync('git', args, { cwd: repo, env: gitEnv(home, seconds), stdio: 'ignore' });
  };
  git(['init', '--quiet', '--initial-branch=main']);
  FIXTURE_COMMITS.forEach((message, index) => {
    git(['commit', '--quiet', '--allow-empty', '-m', message], FIRST_COMMIT_SECONDS + index * 60);
  });
  return {
    path: repo,
    dispose: () => {
      rmSync(base, { recursive: true, force: true });
    },
  };
}

function gitEnv(home: string, seconds: number): NodeJS.ProcessEnv {
  const date = `${String(seconds)} +0200`;
  return {
    PATH: process.env['PATH'],
    HOME: home,
    GIT_CONFIG_GLOBAL: path.join(home, '.gitconfig'),
    GIT_CONFIG_NOSYSTEM: '1',
    LC_ALL: 'C',
    GIT_AUTHOR_NAME: 'Ana Ruiz',
    GIT_AUTHOR_EMAIL: 'ana@example.com',
    GIT_COMMITTER_NAME: 'Ana Ruiz',
    GIT_COMMITTER_EMAIL: 'ana@example.com',
    GIT_AUTHOR_DATE: date,
    GIT_COMMITTER_DATE: date,
  };
}
