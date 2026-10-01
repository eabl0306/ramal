import { describe, expect, it } from 'vitest';

import { SECRET_MASK, maskSecret } from './mask-secret';

describe('maskSecret', () => {
  it('sin llave devuelve vacio para que la interfaz pinte su guion', () => {
    expect(maskSecret('')).toBe('');
    expect(maskSecret('   ')).toBe('');
  });

  it('una llave corta se oculta entera, sin el solape del prototipo', () => {
    expect(maskSecret('abcd')).toBe(SECRET_MASK);
    expect(maskSecret('abcdefghijk')).toBe(SECRET_MASK);
  });

  it('desde 12 caracteres ensena los 4 ultimos', () => {
    expect(maskSecret('abcdefghijkl')).toBe(`${SECRET_MASK}ijkl`);
  });

  it('conserva el prefijo publico del proveedor', () => {
    expect(maskSecret('sk-ant-api03-1234567890abcdWXYZ')).toBe(`sk-ant-${SECRET_MASK}WXYZ`);
    expect(maskSecret('sk-proj-1234567890abcdefWXYZ')).toBe(`sk-proj-${SECRET_MASK}WXYZ`);
    expect(maskSecret('sk-1234567890abWXYZ')).toBe(`sk-${SECRET_MASK}WXYZ`);
  });

  it('no ensena el prefijo si dejaria menos de 8 caracteres ocultos', () => {
    expect(maskSecret('github_pat_12WXYZ')).toBe(`${SECRET_MASK}WXYZ`);
  });
});
