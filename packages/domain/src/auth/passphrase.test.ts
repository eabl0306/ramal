import { describe, expect, it } from 'vitest';

import { MIN_PASSPHRASE_LENGTH, MIN_PASSPHRASE_SCORE, passphraseStrength } from './passphrase';

describe('passphraseStrength: lo que se rechaza', () => {
  it('una frase mas corta que el minimo se rechaza por corta', () => {
    const short = 'a'.repeat(MIN_PASSPHRASE_LENGTH - 1);
    const result = passphraseStrength(short);
    expect(result.ok).toBe(false);
    expect(result.issues).toContain('too-short');
    expect(result.score).toBe(0);
  });

  it('la frase vacia se trata como corta, no como un caso aparte', () => {
    expect(passphraseStrength('').issues).toEqual(['too-short']);
  });

  it('un caracter repetido se rechaza aunque llegue al minimo', () => {
    const result = passphraseStrength('a'.repeat(16));
    expect(result.ok).toBe(false);
    expect(result.issues).toContain('repeated');
    expect(result.score).toBe(0);
  });

  it('una palabra repetida tiene la entropia de la palabra, no la de su longitud', () => {
    const result = passphraseStrength('abcabcabcabcabcabc');
    expect(result.ok).toBe(false);
    expect(result.issues).toContain('repeated');
  });

  it('con muy poca variedad se rechaza por simple', () => {
    const result = passphraseStrength('aaaaabbbbbccc');
    expect(result.ok).toBe(false);
    expect(result.issues).toContain('too-simple');
  });
});

describe('passphraseStrength: lo que se acepta', () => {
  it('una frase larga de solo minusculas vale: la longitud pesa mas que la variedad', () => {
    const result = passphraseStrength('caballo correcto grapa pila');
    expect(result.ok).toBe(true);
    expect(result.issues).toEqual([]);
    expect(result.score).toBeGreaterThanOrEqual(MIN_PASSPHRASE_SCORE);
  });

  it('una frase corta pero variada llega al minimo', () => {
    const result = passphraseStrength('Rudder-7x!qz');
    expect(result.ok).toBe(true);
    expect(result.score).toBeGreaterThanOrEqual(MIN_PASSPHRASE_SCORE);
  });

  it('la puntuacion sube con la longitud y la variedad', () => {
    const plain = passphraseStrength('caballo correcto grapa pila');
    const mixed = passphraseStrength('Caballo Correcto Grapa Pila 42!');
    expect(mixed.score).toBeGreaterThanOrEqual(plain.score);
    expect(mixed.score).toBe(4);
  });
});

describe('passphraseStrength: normalizacion', () => {
  it('mide en NFKC, como la derivacion de la llave', () => {
    // Con la tilde combinada son 12 puntos de codigo sueltos y 11 en NFKC: sin
    // normalizar, una frase de 11 caracteres pasaria por una de 12.
    const combining = 'cafetería12';
    expect(Array.from(combining).length).toBeGreaterThanOrEqual(MIN_PASSPHRASE_LENGTH);
    expect(Array.from(combining.normalize('NFKC')).length).toBeLessThan(MIN_PASSPHRASE_LENGTH);
    expect(passphraseStrength(combining).issues).toContain('too-short');
  });

  it('la misma frase escrita de dos formas unicode da la misma puntuacion', () => {
    const composed = passphraseStrength('Cafetería con leche 42');
    const decomposed = passphraseStrength('Cafetería con leche 42');
    expect(decomposed).toEqual(composed);
  });
});
