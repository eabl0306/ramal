import assert from 'node:assert/strict';
import { after, describe, it } from 'node:test';

import { removeSandboxes, runCheck, sandbox } from './lib/sandbox.mjs';

// Los ${...} de Docker van escapados: aqui son texto, no plantillas de JS.
const SOURCES = {
  'rust-toolchain.toml': '[toolchain]\nchannel = "1.98.1"\n',
  '.nvmrc': '24\n',
  'package.json': { name: 'demo', packageManager: 'pnpm@11.18.0' },
};

const API = [
  'ARG RUST_VERSION=1.98.1',
  'FROM rust:${RUST_VERSION}-slim-trixie AS chef',
  'ARG RUST_VERSION',
  '',
].join('\n');

const WEB = ['ARG NODE_VERSION=24', 'FROM node:${NODE_VERSION}-alpine AS base', ''].join('\n');

function tauri({ rust = '1.98.1', node = '24.21.0', pnpm = '11.18.0' } = {}) {
  return [
    'FROM ubuntu:22.04',
    `ARG RUST_VERSION=${rust}`,
    `ARG NODE_VERSION=${node}`,
    `ARG PNPM_VERSION=${pnpm}`,
    'RUN curl -o /tmp/node.tar.xz "https://nodejs.org/dist/v${NODE_VERSION}/node.tar.xz"',
    'RUN corepack install -g "pnpm@${PNPM_VERSION}"',
    'RUN /tmp/rustup-init -y --default-toolchain "${RUST_VERSION}"',
    '',
  ].join('\n');
}

function repo(overrides = {}) {
  return sandbox({
    ...SOURCES,
    'apps/api/Dockerfile': API,
    // Un .dockerignore se llama como su Dockerfile pero no es uno.
    'apps/api/Dockerfile.dockerignore': '*\n',
    'apps/web/Dockerfile.dev': WEB,
    'tools/docker/tauri-linux.Dockerfile': tauri(),
    ...overrides,
  });
}

function check(root) {
  return runCheck('toolchain-versions.mjs', root);
}

after(removeSandboxes);

describe('toolchain-versions: lo que se acepta', () => {
  it('acepta los Dockerfile a la par con rust-toolchain.toml, .nvmrc y packageManager', () => {
    const { code, output } = check(repo());
    assert.equal(code, 0, output);
    assert.match(output, /3 ficheros revisados/u);
  });

  it('acepta una version de Node completa de la misma mayor que .nvmrc', () => {
    const web = WEB.replace('NODE_VERSION=24', 'NODE_VERSION=24.21.0');
    assert.equal(check(repo({ 'apps/web/Dockerfile.dev': web })).code, 0);
  });
});

describe('toolchain-versions: lo que se rechaza', () => {
  it('rechaza un RUST_VERSION distinto de rust-toolchain.toml (el bump de dependabot)', () => {
    const root = repo({ 'rust-toolchain.toml': '[toolchain]\nchannel = "1.99.0"\n' });
    const { code, output } = check(root);
    assert.equal(code, 1);
    assert.match(output, /apps\/api\/Dockerfile:1 .*RUST_VERSION=1\.98\.1.*1\.99\.0/u);
    assert.match(output, /tools\/docker\/tauri-linux\.Dockerfile:2 .*RUST_VERSION/u);
  });

  it('rechaza un NODE_VERSION de otra mayor que .nvmrc', () => {
    const root = repo({ 'tools/docker/tauri-linux.Dockerfile': tauri({ node: '26.1.0' }) });
    const { code, output } = check(root);
    assert.equal(code, 1);
    assert.match(output, /tauri-linux\.Dockerfile:3 .*NODE_VERSION=26\.1\.0/u);
  });

  it('rechaza un PNPM_VERSION distinto de packageManager', () => {
    const root = repo({ 'tools/docker/tauri-linux.Dockerfile': tauri({ pnpm: '11.17.0' }) });
    const { code, output } = check(root);
    assert.equal(code, 1);
    assert.match(output, /tauri-linux\.Dockerfile:4 .*PNPM_VERSION=11\.17\.0.*11\.18\.0/u);
  });

  it('rechaza una imagen de rust con la version escrita a mano y sin ARG', () => {
    const root = repo({ 'apps/api/Dockerfile': 'FROM rust:1.97.0-slim-trixie\n' });
    const { code, output } = check(root);
    assert.equal(code, 1);
    assert.match(output, /apps\/api\/Dockerfile .*sin ARG RUST_VERSION/u);
    assert.match(output, /apps\/api\/Dockerfile:1 .*no usa \$\{RUST_VERSION\}/u);
  });

  it('rechaza un FROM que no usa el ARG aunque el ARG este al dia', () => {
    const web = 'ARG NODE_VERSION=24\nFROM node:22-alpine\n';
    const { code, output } = check(repo({ 'apps/web/Dockerfile.dev': web }));
    assert.equal(code, 1);
    assert.match(output, /apps\/web\/Dockerfile\.dev:2 .*no usa \$\{NODE_VERSION\}/u);
  });

  it('rechaza un pnpm instalado con la version escrita a mano', () => {
    const web = `${WEB}RUN corepack install -g pnpm@10.0.0\n`;
    const { code, output } = check(repo({ 'apps/web/Dockerfile.dev': web }));
    assert.equal(code, 1);
    assert.match(output, /apps\/web\/Dockerfile\.dev .*sin ARG PNPM_VERSION/u);
  });

  it('falla si no encuentra ningun Dockerfile', () => {
    const { code, output } = check(sandbox(SOURCES));
    assert.equal(code, 1);
    assert.match(output, /ningun Dockerfile/u);
  });

  it('falla si falta una de las versiones de referencia', () => {
    const root = repo({ 'package.json': { name: 'demo' } });
    const { code, output } = check(root);
    assert.equal(code, 1);
    assert.match(output, /package\.json .*packageManager/u);
  });
});
