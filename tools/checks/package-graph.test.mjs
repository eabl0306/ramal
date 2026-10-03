import assert from 'node:assert/strict';
import { after, describe, it } from 'node:test';

import { removeSandboxes, runCheck, sandbox } from './lib/sandbox.mjs';

const WORKSPACE = [
  'packages:',
  '  - "apps/web"',
  '  - "apps/desktop"',
  '  - "apps/vscode"',
  '  - "packages/*"',
  '  - "tools/*"',
  'allowBuilds:',
  '  lefthook: true',
  '',
].join('\n');

const dev = { '@git-rudder/eslint-config': 'workspace:*', '@git-rudder/tsconfig': 'workspace:*' };

function manifest(name, sections = {}) {
  return { name, version: '0.1.0', private: true, ...sections };
}

const CONFIG_DIRS = new Set(['eslint-config', 'tsconfig']);

/** La linea del justfile con las carpetas que recorre just packages-check. */
function justfile(dirs) {
  return `ts_package_dirs := "${dirs.join(' ')}"\n`;
}

function sharedDirs(packages) {
  return Object.keys(packages)
    .map((file) => /^packages\/([^/]+)\/package\.json$/u.exec(file)?.[1])
    .filter((dir) => dir !== undefined && !CONFIG_DIRS.has(dir));
}

/**
 * Por defecto el justfile lista justo los paquetes compartidos que hay, para que
 * cada test mire solo su regla. `justfile: null` monta el repo sin justfile.
 */
function workspace(packages, { justfile: text = justfile(sharedDirs(packages)) } = {}) {
  const files = { 'pnpm-workspace.yaml': WORKSPACE, ...packages };
  return sandbox(text === null ? files : { ...files, justfile: text });
}

function check(root) {
  return runCheck('package-graph.mjs', root);
}

after(removeSandboxes);

describe('package-graph: lo que se acepta', () => {
  it('acepta el monorepo completo con las dependencias de la matriz', () => {
    const w = (names) =>
      Object.fromEntries(names.map((name) => [`@git-rudder/${name}`, 'workspace:*']));
    const root = workspace({
      'packages/eslint-config/package.json': manifest('@git-rudder/eslint-config'),
      'packages/tsconfig/package.json': manifest('@git-rudder/tsconfig'),
      'packages/domain/package.json': manifest('@git-rudder/domain', {
        devDependencies: { ...dev, vitest: '5.0.3' },
      }),
      'packages/i18n/package.json': manifest('@git-rudder/i18n', { devDependencies: dev }),
      'packages/bridge/package.json': manifest('@git-rudder/bridge', { devDependencies: dev }),
      'packages/ui/package.json': manifest('@git-rudder/ui', {
        dependencies: w(['i18n']),
        peerDependencies: { react: '^19.3.0' },
      }),
      'packages/repo-view/package.json': manifest('@git-rudder/repo-view', {
        dependencies: w(['ui', 'domain', 'i18n', 'bridge']),
      }),
      'packages/api-client/package.json': manifest('@git-rudder/api-client', {
        dependencies: { 'openapi-fetch': '0.17.0' },
      }),
      'apps/web/package.json': manifest('@git-rudder/web', {
        dependencies: w(['domain', 'i18n', 'ui', 'api-client']),
      }),
      'apps/desktop/package.json': manifest('@git-rudder/desktop', {
        dependencies: w(['domain', 'i18n', 'ui', 'repo-view', 'bridge']),
      }),
      'apps/vscode/package.json': manifest('git-rudder', {
        dependencies: w(['domain', 'i18n', 'ui', 'repo-view', 'bridge']),
      }),
      'tools/checks/package.json': manifest('@git-rudder/checks', { devDependencies: dev }),
    });
    const { code, output } = check(root);
    assert.equal(code, 0, output);
    assert.match(output, /12 ficheros revisados/u);
  });

  it('funciona cuando todavia faltan paquetes y apps', () => {
    const root = workspace({
      'packages/domain/package.json': manifest('@git-rudder/domain'),
      'packages/eslint-config/package.json': manifest('@git-rudder/eslint-config'),
    });
    assert.equal(check(root).code, 0);
  });

  it('ignora los package.json que no casan con los globs del workspace', () => {
    const root = workspace({
      'examples/demo/package.json': manifest('@git-rudder/demo', { dependencies: { x: '1.0.0' } }),
      'packages/domain/package.json': manifest('@git-rudder/domain'),
    });
    assert.equal(check(root).code, 0);
  });
});

describe('package-graph: lo que se rechaza', () => {
  it('rechaza que repo-view dependa de api-client o de una app', () => {
    const root = workspace({
      'packages/repo-view/package.json': manifest('@git-rudder/repo-view', {
        dependencies: {
          '@git-rudder/api-client': 'workspace:*',
          '@git-rudder/desktop': 'workspace:*',
        },
      }),
    });
    const { code, output } = check(root);
    assert.equal(code, 1);
    assert.match(
      output,
      /"@git-rudder\/repo-view" no puede depender de "@git-rudder\/api-client"/u,
    );
    assert.match(output, /"@git-rudder\/repo-view" no puede depender de "@git-rudder\/desktop"/u);
  });

  it('rechaza que ui dependa de bridge o de repo-view', () => {
    const root = workspace({
      'packages/ui/package.json': manifest('@git-rudder/ui', {
        dependencies: {
          '@git-rudder/bridge': 'workspace:*',
          '@git-rudder/repo-view': 'workspace:*',
        },
      }),
    });
    const { code, output } = check(root);
    assert.equal(code, 1);
    assert.match(output, /"@git-rudder\/ui" no puede depender de "@git-rudder\/bridge"/u);
    assert.match(output, /"@git-rudder\/ui" no puede depender de "@git-rudder\/repo-view"/u);
  });

  it('rechaza dependencias externas en runtime en los paquetes puros', () => {
    const root = workspace({
      'packages/domain/package.json': manifest('@git-rudder/domain', {
        dependencies: { zod: '4.6.5' },
      }),
      'packages/i18n/package.json': manifest('@git-rudder/i18n', {
        peerDependencies: { react: '19' },
      }),
      'packages/bridge/package.json': manifest('@git-rudder/bridge', {
        optionalDependencies: { ws: '8.0.0' },
      }),
    });
    const { code, output } = check(root);
    assert.equal(code, 1);
    assert.match(output, /packages\/domain\/package\.json .*"zod"/u);
    assert.match(output, /packages\/i18n\/package\.json .*"react"/u);
    assert.match(output, /packages\/bridge\/package\.json .*"ws"/u);
  });

  it('rechaza la configuracion compartida fuera de devDependencies', () => {
    const root = workspace({
      'packages/ui/package.json': manifest('@git-rudder/ui', {
        dependencies: { '@git-rudder/tsconfig': 'workspace:*' },
      }),
    });
    const { code, output } = check(root);
    assert.equal(code, 1);
    assert.match(output, /"@git-rudder\/tsconfig" solo puede ir en devDependencies/u);
  });

  it('rechaza un paquete que no esta en la matriz', () => {
    const root = workspace({ 'packages/graph/package.json': manifest('@git-rudder/graph') });
    const { code, output } = check(root);
    assert.equal(code, 1);
    assert.match(output, /paquete desconocido "@git-rudder\/graph"/u);
  });

  it('rechaza una dependencia de un paquete @git-rudder desconocido', () => {
    const root = workspace({
      'apps/web/package.json': manifest('@git-rudder/web', {
        devDependencies: { '@git-rudder/graph': 'workspace:*' },
      }),
    });
    const { code, output } = check(root);
    assert.equal(code, 1);
    assert.match(output, /depende de un paquete desconocido "@git-rudder\/graph"/u);
  });

  it('rechaza que las herramientas dependan de paquetes del producto', () => {
    const root = workspace({
      'tools/checks/package.json': manifest('@git-rudder/checks', {
        devDependencies: { ...dev, '@git-rudder/domain': 'workspace:*' },
      }),
      'packages/domain/package.json': manifest('@git-rudder/domain'),
    });
    const { code, output } = check(root);
    assert.equal(code, 1);
    assert.match(output, /tools\/checks\/package\.json .*"@git-rudder\/domain"/u);
  });

  it('rechaza un paquete interno enlazado sin workspace:', () => {
    const root = workspace({
      'packages/ui/package.json': manifest('@git-rudder/ui', {
        dependencies: { '@git-rudder/i18n': '0.1.0' },
      }),
    });
    const { code, output } = check(root);
    assert.equal(code, 1);
    assert.match(output, /se enlaza con workspace:\*/u);
  });

  it('rechaza un paquete en un directorio que no le toca', () => {
    const root = workspace({ 'packages/design/package.json': manifest('@git-rudder/ui') });
    const { code, output } = check(root);
    assert.equal(code, 1);
    assert.match(output, /tiene que vivir en packages\/ui/u);
  });

  it('rechaza dos paquetes con el mismo nombre', () => {
    const root = workspace({
      'packages/domain/package.json': manifest('@git-rudder/domain'),
      'tools/domain/package.json': manifest('@git-rudder/domain'),
    });
    const { code, output } = check(root);
    assert.equal(code, 1);
    assert.match(output, /nombre repetido con packages\/domain/u);
  });

  it('falla si no hay pnpm-workspace.yaml', () => {
    const root = sandbox({ 'packages/domain/package.json': manifest('@git-rudder/domain') });
    assert.equal(check(root).code, 1);
  });
});

describe('package-graph: la lista de paquetes de just packages-check', () => {
  const domain = { 'packages/domain/package.json': manifest('@git-rudder/domain') };

  it('acepta que la configuracion compartida no este en la lista', () => {
    const root = workspace(
      {
        ...domain,
        'packages/eslint-config/package.json': manifest('@git-rudder/eslint-config'),
        'packages/tsconfig/package.json': manifest('@git-rudder/tsconfig'),
      },
      { justfile: justfile(['domain']) },
    );
    const { code, output } = check(root);
    assert.equal(code, 0, output);
  });

  it('rechaza un paquete compartido que no esta en ts_package_dirs', () => {
    const root = workspace(
      { ...domain, 'packages/i18n/package.json': manifest('@git-rudder/i18n') },
      { justfile: justfile(['domain']) },
    );
    const { code, output } = check(root);
    assert.equal(code, 1);
    assert.match(output, /packages\/i18n\/package\.json {2}no esta en ts_package_dirs/u);
  });

  it('rechaza una carpeta de la lista que no es un paquete (renombrado o borrado)', () => {
    const root = workspace(domain, { justfile: justfile(['domain', 'renamed']) });
    const { code, output } = check(root);
    assert.equal(code, 1);
    assert.match(output, /justfile {2}ts_package_dirs nombra packages\/renamed/u);
  });

  it('rechaza un justfile sin ts_package_dirs y un repo sin justfile', () => {
    for (const text of ['default:\n    @just --list\n', null]) {
      const { code, output } = check(workspace(domain, { justfile: text }));
      assert.equal(code, 1);
      assert.match(output, /justfile {2}falta ts_package_dirs/u);
    }
  });
});
