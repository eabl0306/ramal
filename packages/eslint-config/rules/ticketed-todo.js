/**
 * Regla local: un marcador de trabajo pendiente solo vale si apunta a un ticket,
 * sea una issue de GitHub (#123) o uno del tablero del proyecto (RAM-123).
 *
 * Las palabras clave se construyen por concatenacion a proposito, para que este
 * fichero no dispare su propia regla ni el script tools/checks/todo-without-ticket.mjs,
 * que aplica el mismo criterio a Rust y a cualquier fichero que ESLint no ve.
 * La comparacion es sensible a mayusculas: en castellano "todo" es una palabra normal.
 */
const WORDS = ['TO' + 'DO', 'FIX' + 'ME', 'XX' + 'X', 'HA' + 'CK'];
const MARKER = new RegExp(`\\b(?:${WORDS.join('|')})\\b`, 'u');
const TICKETED = new RegExp(`\\b(?:${WORDS.join('|')})\\((?:#\\d+|RAM-\\d+)\\)`, 'u');

/** @type {import('eslint').Rule.RuleModule} */
export const ticketedTodo = {
  meta: {
    type: 'problem',
    docs: {
      description: 'Exige una referencia a ticket en los marcadores de trabajo pendiente.',
    },
    schema: [],
    messages: {
      missingTicket:
        'Marcador de trabajo pendiente sin ticket. Usa el formato {{example}} o borralo.',
    },
  },
  create(context) {
    const example = `${WORDS[0]}(#123) o ${WORDS[0]}(RAM-123)`;
    return {
      Program() {
        for (const comment of context.sourceCode.getAllComments()) {
          if (!MARKER.test(comment.value)) continue;
          if (TICKETED.test(comment.value)) continue;
          context.report({ loc: comment.loc, messageId: 'missingTicket', data: { example } });
        }
      },
    };
  },
};
