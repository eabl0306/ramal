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

const dev = { '@ramal/eslint-config': 'workspace:*', '@ramal/tsconfig': 'workspace:*' };

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
    const w = (names) => Object.fromEntries(names.map((name) => [`@ramal/${name}`, 'workspace:*']));
    const root = workspace({
      'packages/eslint-config/package.json': manifest('@ramal/eslint-config'),
      'packages/tsconfig/package.json': manifest('@ramal/tsconfig'),
      'packages/domain/package.json': manifest('@ramal/domain', {
        devDependencies: { ...dev, vitest: '5.0.3' },
      }),
      'packages/i18n/package.json': manifest('@ramal/i18n', { devDependencies: dev }),
      'packages/bridge/package.json': manifest('@ramal/bridge', { devDependencies: dev }),
      'packages/ui/package.json': manifest('@ramal/ui', {
        dependencies: w(['i18n']),
        peerDependencies: { react: '^19.3.0' },
      }),
      'packages/repo-view/package.json': manifest('@ramal/repo-view', {
        dependencies: w(['ui', 'domain', 'i18n', 'bridge']),
      }),
      'packages/api-client/package.json': manifest('@ramal/api-client', {
        dependencies: { 'openapi-fetch': '0.17.0' },
      }),
      'apps/web/package.json': manifest('@ramal/web', {
        dependencies: w(['domain', 'i18n', 'ui', 'api-client']),
      }),
      'apps/desktop/package.json': manifest('@ramal/desktop', {
        dependencies: w(['domain', 'i18n', 'ui', 'repo-view', 'bridge']),
      }),
      'apps/vscode/package.json': manifest('ramal', {
        dependencies: w(['domain', 'i18n', 'ui', 'repo-view', 'bridge']),
      }),
      'tools/checks/package.json': manifest('@ramal/checks', { devDependencies: dev }),
    });
    const { code, output } = check(root);
    assert.equal(code, 0, output);
    assert.match(output, /12 ficheros revisados/u);
  });

  it('funciona cuando todavia faltan paquetes y apps', () => {
    const root = workspace({
      'packages/domain/package.json': manifest('@ramal/domain'),
      'packages/eslint-config/package.json': manifest('@ramal/eslint-config'),
    });
    assert.equal(check(root).code, 0);
  });

  it('ignora los package.json que no casan con los globs del workspace', () => {
    const root = workspace({
      'examples/demo/package.json': manifest('@ramal/demo', { dependencies: { x: '1.0.0' } }),
      'packages/domain/package.json': manifest('@ramal/domain'),
    });
    assert.equal(check(root).code, 0);
  });
});

describe('package-graph: lo que se rechaza', () => {
  it('rechaza que repo-view dependa de api-client o de una app', () => {
    const root = workspace({
      'packages/repo-view/package.json': manifest('@ramal/repo-view', {
        dependencies: { '@ramal/api-client': 'workspace:*', '@ramal/desktop': 'workspace:*' },
      }),
    });
    const { code, output } = check(root);
    assert.equal(code, 1);
    assert.match(output, /"@ramal\/repo-view" no puede depender de "@ramal\/api-client"/u);
    assert.match(output, /"@ramal\/repo-view" no puede depender de "@ramal\/desktop"/u);
  });

  it('rechaza que ui dependa de bridge o de repo-view', () => {
    const root = workspace({
      'packages/ui/package.json': manifest('@ramal/ui', {
        dependencies: { '@ramal/bridge': 'workspace:*', '@ramal/repo-view': 'workspace:*' },
      }),
    });
    const { code, output } = check(root);
    assert.equal(code, 1);
    assert.match(output, /"@ramal\/ui" no puede depender de "@ramal\/bridge"/u);
    assert.match(output, /"@ramal\/ui" no puede depender de "@ramal\/repo-view"/u);
  });

  it('rechaza dependencias externas en runtime en los paquetes puros', () => {
    const root = workspace({
      'packages/domain/package.json': manifest('@ramal/domain', { dependencies: { zod: '4.6.5' } }),
      'packages/i18n/package.json': manifest('@ramal/i18n', { peerDependencies: { react: '19' } }),
      'packages/bridge/package.json': manifest('@ramal/bridge', {
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
      'packages/ui/package.json': manifest('@ramal/ui', {
        dependencies: { '@ramal/tsconfig': 'workspace:*' },
      }),
    });
    const { code, output } = check(root);
    assert.equal(code, 1);
    assert.match(output, /"@ramal\/tsconfig" solo puede ir en devDependencies/u);
  });

  it('rechaza un paquete que no esta en la matriz', () => {
    const root = workspace({ 'packages/graph/package.json': manifest('@ramal/graph') });
    const { code, output } = check(root);
    assert.equal(code, 1);
    assert.match(output, /paquete desconocido "@ramal\/graph"/u);
  });

  it('rechaza una dependencia de un paquete @ramal desconocido', () => {
    const root = workspace({
      'apps/web/package.json': manifest('@ramal/web', {
        devDependencies: { '@ramal/graph': 'workspace:*' },
      }),
    });
    const { code, output } = check(root);
    assert.equal(code, 1);
    assert.match(output, /depende de un paquete desconocido "@ramal\/graph"/u);
  });

  it('rechaza que las herramientas dependan de paquetes del producto', () => {
    const root = workspace({
      'tools/checks/package.json': manifest('@ramal/checks', {
        devDependencies: { ...dev, '@ramal/domain': 'workspace:*' },
      }),
      'packages/domain/package.json': manifest('@ramal/domain'),
    });
    const { code, output } = check(root);
    assert.equal(code, 1);
    assert.match(output, /tools\/checks\/package\.json .*"@ramal\/domain"/u);
  });

  it('rechaza un paquete interno enlazado sin workspace:', () => {
    const root = workspace({
      'packages/ui/package.json': manifest('@ramal/ui', {
        dependencies: { '@ramal/i18n': '0.1.0' },
      }),
    });
    const { code, output } = check(root);
    assert.equal(code, 1);
    assert.match(output, /se enlaza con workspace:\*/u);
  });

  it('rechaza un paquete en un directorio que no le toca', () => {
    const root = workspace({ 'packages/design/package.json': manifest('@ramal/ui') });
    const { code, output } = check(root);
    assert.equal(code, 1);
    assert.match(output, /tiene que vivir en packages\/ui/u);
  });

  it('rechaza dos paquetes con el mismo nombre', () => {
    const root = workspace({
      'packages/domain/package.json': manifest('@ramal/domain'),
      'tools/domain/package.json': manifest('@ramal/domain'),
    });
    const { code, output } = check(root);
    assert.equal(code, 1);
    assert.match(output, /nombre repetido con packages\/domain/u);
  });

  it('falla si no hay pnpm-workspace.yaml', () => {
    const root = sandbox({ 'packages/domain/package.json': manifest('@ramal/domain') });
    assert.equal(check(root).code, 1);
  });
});

describe('package-graph: la lista de paquetes de just packages-check', () => {
  const domain = { 'packages/domain/package.json': manifest('@ramal/domain') };

  it('acepta que la configuracion compartida no este en la lista', () => {
    const root = workspace(
      {
        ...domain,
        'packages/eslint-config/package.json': manifest('@ramal/eslint-config'),
        'packages/tsconfig/package.json': manifest('@ramal/tsconfig'),
      },
      { justfile: justfile(['domain']) },
    );
    const { code, output } = check(root);
    assert.equal(code, 0, output);
  });

  it('rechaza un paquete compartido que no esta en ts_package_dirs', () => {
    const root = workspace(
      { ...domain, 'packages/i18n/package.json': manifest('@ramal/i18n') },
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
