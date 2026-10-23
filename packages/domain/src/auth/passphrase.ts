/**
 * Fortaleza de la frase de sincronizacion de la boveda.
 *
 * **La frase es la unica defensa de la boveda.** El servidor no la ve nunca, asi
 * que no puede exigir nada: quien robe la base de datos se lleva las envolturas y
 * puede probar frases sin conexion, y la defensa de fondo son el coste de Argon2id
 * y la longitud de la frase
 * ([ADR 0012](../../../../docs/adr/0012-boveda-de-extremo-a-extremo.md)). Por eso
 * el minimo lo impone el cliente, y por eso esto no es un medidor decorativo: lo
 * que no pase de aqui no se envuelve.
 *
 * Como el resto de `@git-rudder/domain`, devuelve codigos y no textos: el texto
 * depende del idioma y lo pone la interfaz con `@git-rudder/i18n`.
 */

/** Minimo de la frase, en caracteres (ADR 0012). */
export const MIN_PASSPHRASE_LENGTH = 12;

/** Puntuacion minima que se acepta para envolver la boveda. */
export const MIN_PASSPHRASE_SCORE = 2;

export type PassphraseIssue =
  /** No llega al minimo de caracteres. */
  | 'too-short'
  /** Un solo tipo de caracter, o demasiado poca variedad para su longitud. */
  | 'too-simple'
  /** Una palabra o un caracter repetidos ("abcabcabcabc", "aaaaaaaaaaaa"). */
  | 'repeated';

/** 0 es inservible y 4 es de sobra; a partir de `MIN_PASSPHRASE_SCORE` se acepta. */
export type PassphraseScore = 0 | 1 | 2 | 3 | 4;

export interface PassphraseStrength {
  readonly score: PassphraseScore;
  /** Por que no vale, en el orden en que conviene ensenarlos. Vacio si vale. */
  readonly issues: readonly PassphraseIssue[];
  readonly ok: boolean;
}

/**
 * La frase se mide normalizada en NFKC, que es como la normaliza la derivacion de
 * la llave: asi "cafe" escrito con una tilde combinada cuenta los mismos
 * caracteres que la persona ve en pantalla, y no uno mas.
 */
function normalize(raw: string): string {
  return raw.normalize('NFKC');
}

/**
 * Los caracteres de la frase, en grafemas: es la unidad que la persona cuenta en
 * pantalla, la misma que usa el contador del resumen del commit. Un emoji con
 * modificador cuenta como uno, no como los tres puntos de codigo que ocupa.
 */
const graphemes = new Intl.Segmenter(undefined, { granularity: 'grapheme' });

function toGraphemes(phrase: string): string[] {
  return Array.from(graphemes.segment(phrase), (segment) => segment.segment);
}

const CLASSES = [/\p{Ll}/u, /\p{Lu}/u, /\p{N}/u, /[^\p{L}\p{N}]/u];

function classCount(phrase: string): number {
  return CLASSES.filter((pattern) => pattern.test(phrase)).length;
}

/**
 * Una frase hecha de un trozo repetido tiene la entropia del trozo, no la de su
 * longitud: "abcabcabcabc" son doce caracteres y tres de informacion. Se busca el
 * trozo mas corto que, repetido, da la frase entera.
 */
function isRepeated(chars: readonly string[]): boolean {
  const phrase = chars.join('');
  for (let size = 1; size <= Math.floor(chars.length / 2); size += 1) {
    if (chars.length % size !== 0) continue;
    const unit = chars.slice(0, size).join('');
    if (unit.repeat(chars.length / size) === phrase) return true;
  }
  return false;
}

/** Caracteres distintos: "aaaaabbbbbcc" es larga y tiene tres. */
function distinctCount(chars: readonly string[]): number {
  return new Set(chars).size;
}

function scoreOf(length: number, classes: number, distinct: number): PassphraseScore {
  if (length < MIN_PASSPHRASE_LENGTH || distinct < 4) return 0;
  // Una frase larga vale aunque sea todo minusculas: "caballo correcto grapa pila"
  // es mejor que "P@ss1!". La longitud pesa mas que la variedad, a proposito.
  const points = (length >= 20 ? 2 : 1) + (classes >= 3 ? 2 : classes - 1);
  return Math.max(0, Math.min(4, points)) as PassphraseScore;
}

/**
 * Mide la frase y dice si sirve. Una frase vacia da `too-short`, no un caso
 * aparte: al crear la boveda el campo empieza vacio y la interfaz ensena el
 * mismo aviso hasta que llega al minimo.
 */
export function passphraseStrength(raw: string): PassphraseStrength {
  const phrase = normalize(raw);
  const chars = toGraphemes(phrase);
  const issues: PassphraseIssue[] = [];
  if (chars.length < MIN_PASSPHRASE_LENGTH) {
    issues.push('too-short');
    return { score: 0, issues, ok: false };
  }
  if (isRepeated(chars)) {
    issues.push('repeated');
    return { score: 0, issues, ok: false };
  }
  const score = scoreOf(chars.length, classCount(phrase), distinctCount(chars));
  if (score < MIN_PASSPHRASE_SCORE) issues.push('too-simple');
  return { score, issues, ok: issues.length === 0 };
}
