import { describe, expect, it } from 'vitest';

import { parseRemoteUrl, remoteRepoName } from './remote-url';

/**
 * Los mismos vectores que la tabla de normalizacion de remotos de
 * api-surface.md (ADR 0019). La `remoteKey` la calcula el servidor en Rust; lo
 * que se comprueba aqui es que de estas URL salen el mismo host, propietario y
 * nombre, para que las dos lean las mismas direcciones de la misma forma.
 */
const SHARED_VECTORS = [
  {
    url: 'https://github.com/ACME/Web-App.git',
    host: 'github.com',
    owner: 'ACME',
    name: 'Web-App',
  },
  { url: 'git@github.com:acme/web-app.git', host: 'github.com', owner: 'acme', name: 'web-app' },
  {
    url: 'ssh://git@gitlab.empresa.com:2222/equipo/api',
    host: 'gitlab.empresa.com:2222',
    owner: 'equipo',
    name: 'api',
  },
  {
    url: 'https://marta@dev.azure.com/banca/core/_git/pagos',
    host: 'dev.azure.com',
    owner: 'banca/core',
    name: 'pagos',
  },
  {
    url: 'git@ssh.dev.azure.com:v3/banca/core/pagos',
    host: 'dev.azure.com',
    owner: 'banca/core',
    name: 'pagos',
  },
] as const;

describe('parseRemoteUrl: los vectores que comparte con la normalizacion', () => {
  it.each(SHARED_VECTORS)('lee $url', ({ url, host, owner, name }) => {
    expect(parseRemoteUrl(url)).toEqual({ host, owner, name });
  });
});

describe('parseRemoteUrl: formas de URL', () => {
  it('lee https, http y git con y sin .git', () => {
    const expected = { host: 'github.com', owner: 'acme', name: 'web-app' };
    expect(parseRemoteUrl('https://github.com/acme/web-app')).toEqual(expected);
    expect(parseRemoteUrl('http://github.com/acme/web-app.git')).toEqual(expected);
    expect(parseRemoteUrl('git://github.com/acme/web-app.git')).toEqual(expected);
  });

  it('la barra final y el .git no cambian el resultado', () => {
    const expected = { host: 'github.com', owner: 'acme', name: 'web-app' };
    expect(parseRemoteUrl('https://github.com/acme/web-app/')).toEqual(expected);
    expect(parseRemoteUrl('https://github.com/acme/web-app.git/')).toEqual(expected);
    expect(parseRemoteUrl('git@github.com:acme/web-app.git/')).toEqual(expected);
    expect(parseRemoteUrl('  https://github.com/acme/web-app.git  ')).toEqual(expected);
  });

  it('quita el usuario y el puerto de por defecto del host', () => {
    expect(parseRemoteUrl('https://marta@github.com/acme/web-app')?.host).toBe('github.com');
    expect(parseRemoteUrl('https://github.com:443/acme/web-app')?.host).toBe('github.com');
    expect(parseRemoteUrl('ssh://git@github.com:22/acme/web-app')?.host).toBe('github.com');
    expect(parseRemoteUrl('ssh://git@github.com:2222/acme/web-app')?.host).toBe('github.com:2222');
  });

  it('acepta el atajo propietario/repo que se escribe a mano', () => {
    expect(parseRemoteUrl('acme/web-app')).toEqual({ host: '', owner: 'acme', name: 'web-app' });
  });

  it('pasa el host a minusculas y deja la ruta como esta', () => {
    expect(parseRemoteUrl('https://GitHub.COM/ACME/Web-App')).toEqual({
      host: 'github.com',
      owner: 'ACME',
      name: 'Web-App',
    });
  });

  it('lee subgrupos de GitLab como un propietario con barras', () => {
    expect(parseRemoteUrl('https://gitlab.com/grupo/sub/equipo/api.git')).toEqual({
      host: 'gitlab.com',
      owner: 'grupo/sub/equipo',
      name: 'api',
    });
  });
});

describe('parseRemoteUrl: lo que no se entiende', () => {
  it.each([
    ['vacio', ''],
    ['solo espacios', '   '],
    ['una ruta local', '/home/marta/proyecto'],
    ['una ruta relativa', './proyecto'],
    ['un host sin propietario', 'https://github.com/web-app'],
    ['la forma corta sin propietario', 'git@github.com:web-app.git'],
    ['una URL rota', 'https://'],
  ])('devuelve null con %s', (_caso, url) => {
    expect(parseRemoteUrl(url)).toBeNull();
  });
});

describe('remoteRepoName', () => {
  it('es el nombre que se propone al clonar', () => {
    expect(remoteRepoName('git@github.com:acme/web-app.git')).toBe('web-app');
    expect(remoteRepoName('https://github.com/acme/web-app/')).toBe('web-app');
  });

  it('da cadena vacia cuando la URL no se entiende, en vez de inventarse un nombre', () => {
    expect(remoteRepoName('/home/marta/proyecto')).toBe('');
    expect(remoteRepoName('')).toBe('');
  });
});
