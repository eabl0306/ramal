import assert from 'node:assert/strict';
import { after, describe, it } from 'node:test';

import { slug } from './docs-links.mjs';
import { removeSandboxes, runCheck, sandbox } from './lib/sandbox.mjs';

after(removeSandboxes);

const ADR = '# ADR 0019: Sincronizacion\n\n- Estado: aceptado\n';

function repo(files) {
  return sandbox({ 'docs/adr/0019-sincronizacion.md': ADR, ...files });
}

describe('docs-links: lo que se acepta', () => {
  it('acepta un enlace a un fichero y a una ancla que existen', () => {
    const root = repo({
      'docs/architecture.md': '# Arquitectura\n\n## 11. Indice de decisiones\n',
      'docs/api-surface.md':
        'Ver [el indice](architecture.md#11-indice-de-decisiones) y el [ADR 0019](adr/0019-sincronizacion.md).\n',
    });
    const { code, output } = runCheck('docs-links.mjs', root);
    assert.equal(code, 0, output);
  });

  it('acepta una ancla del propio documento y un enlace que sube de carpeta', () => {
    const root = repo({
      'docs/architecture.md': '# Arquitectura\n\n## El motor\n\nVer [el motor](#el-motor).\n',
      'docs/adr/0019-sincronizacion.md': `${ADR}\nVer [la arquitectura](../architecture.md#el-motor).\n`,
    });
    assert.equal(runCheck('docs-links.mjs', root).code, 0);
  });

  it('no mira las URL externas ni los enlaces de dentro de un bloque de codigo', () => {
    const root = repo({
      'docs/architecture.md': [
        '# Arquitectura',
        '',
        'Un [sitio](https://git-rudder.com/nada) y un [correo](mailto:hola@git-rudder.com).',
        '',
        '```md',
        'Un [enlace de ejemplo](no-existe.md) y el ADR 9999.',
        '```',
        '',
      ].join('\n'),
    });
    assert.equal(runCheck('docs-links.mjs', root).code, 0);
  });

  it('distingue dos titulos iguales como GitHub, con sufijo', () => {
    const root = repo({
      'docs/architecture.md': '# Arquitectura\n\n## Notas\n\n## Notas\n',
      'docs/api-surface.md':
        'La [primera](architecture.md#notas) y la [segunda](architecture.md#notas-1).\n',
    });
    assert.equal(runCheck('docs-links.mjs', root).code, 0);
  });
});

describe('docs-links: lo que se rechaza', () => {
  it('rechaza un enlace a un fichero que no existe', () => {
    const root = repo({ 'docs/architecture.md': '# A\n\nVer [lo que falta](no-existe.md).\n' });
    const { code, output } = runCheck('docs-links.mjs', root);
    assert.equal(code, 1);
    assert.match(output, /docs\/architecture\.md:3\s+enlace roto: no-existe\.md/u);
  });

  it('rechaza una ancla que ningun titulo genera', () => {
    const root = repo({
      'docs/architecture.md': '# Arquitectura\n\n## El motor\n',
      'docs/api-surface.md': 'Ver [lo que falta](architecture.md#el-motorcito).\n',
    });
    const { code, output } = runCheck('docs-links.mjs', root);
    assert.equal(code, 1);
    assert.match(output, /ancla inexistente: architecture\.md#el-motorcito/u);
  });

  it('rechaza una ancla rota dentro del propio documento', () => {
    const root = repo({ 'docs/architecture.md': '# Arquitectura\n\nVer [esto](#no-esta).\n' });
    assert.equal(runCheck('docs-links.mjs', root).code, 1);
  });

  it('rechaza citar un ADR que no existe', () => {
    const root = repo({ 'docs/architecture.md': '# A\n\nLo dice el ADR 0042.\n' });
    const { code, output } = runCheck('docs-links.mjs', root);
    assert.equal(code, 1);
    assert.match(output, /ADR 0042 citado y no existe/u);
  });

  it('tambien mira el Markdown de la raiz', () => {
    const root = repo({ 'README.md': '# Git Rudder\n\nVer [los docs](docs/no-existe.md).\n' });
    assert.equal(runCheck('docs-links.mjs', root).code, 1);
  });

  it('no mira docs/design/handoff, que es la entrega del diseno', () => {
    const root = repo({ 'docs/design/handoff/README.md': '# Maquetas\n\n[roto](no-existe.md)\n' });
    assert.equal(runCheck('docs-links.mjs', root).code, 0);
  });
});

describe('docs-links: anclas al estilo de GitHub', () => {
  it('pasa el titulo a minusculas, cambia los espacios y quita la puntuacion', () => {
    assert.equal(slug('## 6.5 El protocolo del motor'), '65-el-protocolo-del-motor');
    assert.equal(
      slug('Identidad: el remoto normalizado, nunca la ruta'),
      'identidad-el-remoto-normalizado-nunca-la-ruta',
    );
  });

  it('conserva los acentos y quita el formato de Markdown', () => {
    assert.equal(slug('Qué se **sincroniza** y `que` no'), 'qué-se-sincroniza-y-que-no');
    assert.equal(slug('El [ADR 0019](adr/0019-x.md) manda'), 'el-adr-0019-manda');
  });
});
