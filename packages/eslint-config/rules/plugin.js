import { noJsxLiteralText } from './no-jsx-literal-text.js';
import { noRawColors } from './no-raw-colors.js';
import { ticketedTodo } from './ticketed-todo.js';

/**
 * Plugin minimo del monorepo, solo para reglas que no existen upstream. Se
 * registra con la clave `ramal` en todas las capas que lo usan, siempre con este
 * mismo objeto: ESLint se queja si una misma clave apunta a dos plugins distintos.
 */
export const ramalPlugin = {
  meta: { name: '@ramal/eslint-config', version: '0.1.0' },
  rules: {
    'no-jsx-literal-text': noJsxLiteralText,
    'no-raw-colors': noRawColors,
    'ticketed-todo': ticketedTodo,
  },
};
