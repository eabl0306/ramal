import type { Catalog } from './catalog';
import type { PARAMS } from './generated/params';

/**
 * Tipos de las claves y de sus parametros.
 *
 * Las claves salen de la forma del JSON castellano. Los parametros no pueden
 * salir del JSON, porque TypeScript importa sus textos como `string` y no como
 * literales; los extrae scripts/generate-params.mjs a src/generated/params.ts y
 * un test falla si ese fichero no esta al dia.
 */

type Leaves<T, P extends string = ''> = {
  [K in keyof T & string]: T[K] extends string ? `${P}${K}` : Leaves<T[K], `${P}${K}.`>;
}[keyof T & string];

/** Las formas plurales son sufijos de la misma clave, como en i18next. */
export const PLURAL_SUFFIXES = ['zero', 'one', 'two', 'few', 'many', 'other'] as const;

type StripPlural<K extends string> = K extends `${infer B}_zero`
  ? B
  : K extends `${infer B}_one`
    ? B
    : K extends `${infer B}_two`
      ? B
      : K extends `${infer B}_few`
        ? B
        : K extends `${infer B}_many`
          ? B
          : K extends `${infer B}_other`
            ? B
            : K;

/** Toda clave que se puede traducir; las plurales sin su sufijo. */
export type TranslationKey = StripPlural<Leaves<Catalog>>;

type ParamsTable = typeof PARAMS;

export type ParamName<K extends TranslationKey> = K extends keyof ParamsTable
  ? ParamsTable[K][number]
  : never;

export type TranslationParams<K extends TranslationKey> = Readonly<
  Record<ParamName<K>, string | number>
>;

/** Sin parametros no se pasa nada; con parametros, todos y con su nombre. */
export type TranslateArgs<K extends TranslationKey> = [ParamName<K>] extends [never]
  ? []
  : [params: TranslationParams<K>];

export type Translate = <K extends TranslationKey>(key: K, ...args: TranslateArgs<K>) => string;

/** Las claves que no llevan parametros, para props que reciben una clave suelta. */
export type PlainKey = {
  [K in TranslationKey]: [ParamName<K>] extends [never] ? K : never;
}[TranslationKey];
