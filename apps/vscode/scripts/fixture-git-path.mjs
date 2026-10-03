// Perfil de usuario de los tests de integracion con un `git.path` propio: un
// enlace al git de la maquina en una carpeta temporal. Asi el git que usa VS Code
// no es el primero del PATH, y el test puede comprobar que el sidecar recibe el de
// VS Code y no el que encontraria por su cuenta. En Windows no se crea el enlace
// (pide permisos) y el perfil va sin ajustes.
import { execFileSync } from 'node:child_process';
import { mkdirSync, mkdtempSync, symlinkSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';

/** El git del PATH, como lo encontraria el sidecar sin `--git-path`. */
function gitOnPath() {
  return execFileSync('sh', ['-c', 'command -v git'], { encoding: 'utf8' }).trim();
}

/**
 * Crea el perfil y devuelve su carpeta de datos de usuario y el `git.path` que
 * configura (vacio si no configura ninguno).
 */
export function createGitPathProfile() {
  const root = mkdtempSync(path.join(tmpdir(), 'git-rudder-vscode-profile-'));
  const userDir = path.join(root, 'user-data', 'User');
  mkdirSync(userDir, { recursive: true });
  let gitPath = '';
  if (process.platform !== 'win32') {
    mkdirSync(path.join(root, 'bin'));
    gitPath = path.join(root, 'bin', 'git');
    symlinkSync(gitOnPath(), gitPath);
  }
  const settings = gitPath === '' ? {} : { 'git.path': gitPath };
  writeFileSync(path.join(userDir, 'settings.json'), `${JSON.stringify(settings, null, 2)}\n`);
  return { userDataDir: path.join(root, 'user-data'), gitPath };
}
