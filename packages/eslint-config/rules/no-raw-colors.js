/**
 * Regla local: los colores salen de los tokens de @git-rudder/ui, nunca de un literal.
 *
 * Git Rudder tiene tema claro y oscuro, y el grafo de ramas usa una paleta propia. Un
 * color escrito a mano en un componente no cambia con el tema y rompe el contraste
 * en uno de los dos. La unica carpeta que puede escribir colores es la que define
 * los temas (packages/ui/src/themes); el resto usa var(--...) o los tokens.
 *
 * Se detectan hexadecimales (#fff, #1e1e2e, #1e1e2eff) y las funciones de color de
 * CSS (rgb, hsl, hwb, lab, lch, oklab, oklch). Un hexadecimal de solo cifras, como
 * "#123", es tambien la forma de citar una issue de GitHub, asi que solo cuenta si
 * es todo el texto o si va dentro de un valor CSS: detras de "(" o ",", despues de
 * los dos puntos de una declaracion ("color: #333") o detras de una medida o un
 * estilo de borde ("1px solid #333", "0 0 4px #000").
 */
const COLOR_FUNCTION = /\b(?:rgba?|hsla?|hwb|lab|lch|oklab|oklch)\(/iu;
const HEX = /#([\da-f]{8}|[\da-f]{6}|[\da-f]{3,4})(?![\w-])/giu;
const CSS_VALUE_BEFORE =
  /(?:[(,]\s*|:[^;{}]*|(?:\d(?:px|r?em|%)?|solid|dashed|dotted|double|inset)\s+)$/u;
const HAS_HEX_LETTER = /[a-f]/iu;
const DEFAULT_ALLOW = ['packages/ui/src/themes/'];

function looksLikeHexColor(text) {
  for (const match of text.matchAll(HEX)) {
    const before = text.slice(0, match.index);
    if (/[\w&]$/u.test(before)) continue;
    if (HAS_HEX_LETTER.test(match[1] ?? '')) return true;
    if (text.trim() === match[0] || CSS_VALUE_BEFORE.test(before)) return true;
  }
  return false;
}

export function containsRawColor(text) {
  return COLOR_FUNCTION.test(text) || looksLikeHexColor(text);
}

function isAllowedFile(filename, allow) {
  const normalized = filename.split('\\').join('/');
  return allow.some((prefix) => normalized.startsWith(prefix) || normalized.includes(`/${prefix}`));
}

/** @type {import('eslint').Rule.RuleModule} */
export const noRawColors = {
  meta: {
    type: 'problem',
    docs: { description: 'Prohibe colores escritos a mano fuera de los temas de @git-rudder/ui.' },
    schema: [
      {
        type: 'object',
        properties: { allow: { type: 'array', items: { type: 'string' } } },
        additionalProperties: false,
      },
    ],
    messages: {
      rawColor: 'Color escrito a mano. Los colores salen de los tokens de @git-rudder/ui.',
    },
  },
  create(context) {
    const allow = context.options[0]?.allow ?? DEFAULT_ALLOW;
    if (isAllowedFile(context.physicalFilename, allow)) return {};
    const check = (node, text) => {
      if (containsRawColor(text)) context.report({ node, messageId: 'rawColor' });
    };
    return {
      Literal(node) {
        if (typeof node.value === 'string') check(node, node.value);
      },
      TemplateElement(node) {
        check(node, node.value.cooked ?? node.value.raw);
      },
    };
  },
};
