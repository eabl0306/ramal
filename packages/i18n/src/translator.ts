import { type InterpolationValues, interpolate } from './interpolate';
import type { Translate, TranslateArgs, TranslationKey } from './keys';
import { FALLBACK_LOCALE, type Locale } from './locale';
import { messagesFor } from './messages';
import { pluralTemplate } from './plural';

/**
 * El traductor. Es una funcion pura sobre los diccionarios, sin React, para que
 * la usen igual los componentes (a traves de useT en @ramal/ui), el extension
 * host de VS Code y cualquier script.
 */

interface LocaleTools {
  readonly messages: ReadonlyMap<string, string>;
  readonly rules: Intl.PluralRules;
  readonly numbers: Intl.NumberFormat;
}

function toolsFor(locale: Locale): LocaleTools {
  return {
    messages: messagesFor(locale),
    rules: new Intl.PluralRules(locale),
    numbers: new Intl.NumberFormat(locale),
  };
}

function templateIn(tools: LocaleTools, key: string, count: unknown): string | undefined {
  const direct = tools.messages.get(key);
  if (direct !== undefined) return direct;
  if (typeof count === 'number') return pluralTemplate(tools.messages, key, count, tools.rules);
  return tools.messages.get(`${key}_other`);
}

/**
 * `t("toast.pushed", { branch: "main" })`. Si la clave faltara en el idioma (el
 * tipado y el test de paridad lo impiden) se usa el ingles, y si tampoco esta,
 * la propia clave, para que el fallo se vea.
 */
export function createTranslator(locale: Locale): Translate {
  const primary = toolsFor(locale);
  const fallback = locale === FALLBACK_LOCALE ? primary : toolsFor(FALLBACK_LOCALE);
  return function translate<K extends TranslationKey>(key: K, ...args: TranslateArgs<K>): string {
    const values = (args as readonly unknown[])[0] as InterpolationValues | undefined;
    const count = values?.['count'];
    const own = templateIn(primary, key, count);
    const tools = own === undefined ? fallback : primary;
    const template = own ?? templateIn(fallback, key, count) ?? key;
    return interpolate(template, values, (value) => tools.numbers.format(value));
  };
}
