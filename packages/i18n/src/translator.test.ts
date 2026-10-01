import { describe, expect, it } from 'vitest';

import { interpolate } from './interpolate';
import type { PlainKey, TranslationKey } from './keys';
import { pluralTemplate } from './plural';
import { createTranslator } from './translator';

const es = createTranslator('es');
const en = createTranslator('en');

describe('createTranslator', () => {
  it('traduce una clave sin parametros en cada idioma', () => {
    expect(es('common.save')).toBe('Guardar');
    expect(en('common.save')).toBe('Save');
  });

  it('sustituye los parametros con nombre', () => {
    expect(es('toast.pushed', { branch: 'main' })).toBe('Push a main completado');
    expect(en('menu.merge', { source: 'hotfix/login', target: 'develop' })).toBe(
      'Merge hotfix/login into develop…',
    );
  });

  it('elige la forma plural con el recuento', () => {
    expect(es('repos.results', { count: 1 })).toBe('1 repositorio');
    expect(es('repos.results', { count: 2 })).toBe('2 repositorios');
    expect(es('repos.results', { count: 0 })).toBe('0 repositorios');
    expect(en('repos.results', { count: 1 })).toBe('1 repository');
    expect(en('staging.commitButton', { count: 3, branch: 'main' })).toBe('Commit 3 files to main');
  });

  it('formatea los numeros con el idioma y cae a _other en la categoria many', () => {
    expect(es('repos.results', { count: 12345 })).toBe('12.345 repositorios');
    expect(en('repos.results', { count: 12345 })).toBe('12,345 repositories');
    expect(es('repos.results', { count: 1000000 })).toBe('1.000.000 repositorios');
  });

  it('una clave que no existe devuelve la propia clave para que se vea', () => {
    const missing = 'no.existe' as TranslationKey;
    expect(es(missing as PlainKey)).toBe('no.existe');
  });

  it('el tipado exige los parametros correctos', () => {
    // @ts-expect-error -- falta el parametro branch
    expect(es('toast.pushed')).toBe('Push a {branch} completado');
    // @ts-expect-error -- el parametro se llama branch, no b
    expect(es('toast.pushed', { b: 'main' })).toBe('Push a {branch} completado');
    // @ts-expect-error -- una clave sin parametros no acepta parametros
    expect(es('common.save', { count: 1 })).toBe('Guardar');
  });
});

describe('pluralTemplate', () => {
  const rules = new Intl.PluralRules('es');
  const messages = new Map([
    ['k_zero', 'ninguno'],
    ['k_one', 'uno'],
    ['k_other', 'varios'],
  ]);

  it('usa _zero con cero cuando existe', () => {
    expect(pluralTemplate(messages, 'k', 0, rules)).toBe('ninguno');
    expect(pluralTemplate(new Map([['k_other', 'varios']]), 'k', 0, rules)).toBe('varios');
  });

  it('usa la categoria del idioma o _other', () => {
    expect(pluralTemplate(messages, 'k', 1, rules)).toBe('uno');
    expect(pluralTemplate(messages, 'k', 7, rules)).toBe('varios');
    expect(pluralTemplate(messages, 'otra', 7, rules)).toBeUndefined();
  });
});

describe('interpolate', () => {
  it('deja a la vista un marcador sin valor', () => {
    expect(interpolate('{a} y {b}', { a: 'uno' }, String)).toBe('uno y {b}');
    expect(interpolate('sin {nada}', undefined, String)).toBe('sin {nada}');
  });
});
