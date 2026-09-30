/**
 * Regla local: ningun texto visible se escribe a mano en el JSX.
 *
 * Ramal se traduce entero desde @ramal/i18n. Un texto literal en un componente es
 * un texto que se queda sin traducir y que nadie ve hasta que alguien cambia de
 * idioma. Por eso se rechaza cualquier nodo de texto con letras y los atributos que
 * el usuario lee (title, aria-label, placeholder, alt, label) con un string fijo.
 *
 * Solo cuentan los textos con alguna letra (\p{L}, en cualquier alfabeto): la
 * puntuacion, los numeros y los separadores sueltos no se traducen. Un alt vacio
 * sigue valiendo, porque es la forma correcta de marcar una imagen decorativa.
 */
const LETTER = /\p{L}/u;
const DEFAULT_ATTRIBUTES = ['title', 'aria-label', 'placeholder', 'alt', 'label'];

/** Devuelve los nodos de la expresion que producen un texto fijo conocido. */
function staticTextNodes(node) {
  if (node == null) return [];
  switch (node.type) {
    case 'Literal':
      return typeof node.value === 'string' ? [{ node, text: node.value }] : [];
    case 'TemplateLiteral':
      if (node.expressions.length > 0) return [];
      return [{ node, text: node.quasis.map((quasi) => quasi.value.cooked ?? '').join('') }];
    case 'JSXExpressionContainer':
      return staticTextNodes(node.expression);
    case 'ConditionalExpression':
      return [...staticTextNodes(node.consequent), ...staticTextNodes(node.alternate)];
    case 'LogicalExpression':
      return [...staticTextNodes(node.left), ...staticTextNodes(node.right)];
    default:
      return [];
  }
}

function attributeName(node) {
  return node.name.type === 'JSXIdentifier' ? node.name.name : null;
}

function isJsxChild(node) {
  return node.parent?.type === 'JSXElement' || node.parent?.type === 'JSXFragment';
}

/** @type {import('eslint').Rule.RuleModule} */
export const noJsxLiteralText = {
  meta: {
    type: 'problem',
    docs: { description: 'Prohibe textos visibles escritos a mano en el JSX.' },
    schema: [
      {
        type: 'object',
        properties: { attributes: { type: 'array', items: { type: 'string' } } },
        additionalProperties: false,
      },
    ],
    messages: {
      literalText:
        'Texto visible escrito a mano: "{{text}}". Los textos visibles salen de @ramal/i18n.',
      literalAttribute:
        'El atributo {{name}} lleva un texto fijo. Los textos visibles salen de @ramal/i18n.',
    },
  },
  create(context) {
    const attributes = new Set(context.options[0]?.attributes ?? DEFAULT_ATTRIBUTES);
    const reportText = (node, text) => {
      const shown = text.trim().slice(0, 40);
      context.report({ node, messageId: 'literalText', data: { text: shown } });
    };
    return {
      JSXText(node) {
        if (LETTER.test(node.value)) reportText(node, node.value);
      },
      JSXExpressionContainer(node) {
        if (!isJsxChild(node)) return;
        for (const found of staticTextNodes(node.expression)) {
          if (LETTER.test(found.text)) reportText(found.node, found.text);
        }
      },
      JSXAttribute(node) {
        const name = attributeName(node);
        if (name === null || !attributes.has(name)) return;
        const texts = staticTextNodes(node.value).filter((found) => LETTER.test(found.text));
        for (const found of texts) {
          context.report({ node: found.node, messageId: 'literalAttribute', data: { name } });
        }
      },
    };
  },
};
