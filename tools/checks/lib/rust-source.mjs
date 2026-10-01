/**
 * Borra de un fuente de Rust los comentarios y el contenido de los literales de
 * texto y de caracter, cambiandolos por espacios y conservando los saltos de
 * linea. Asi una busqueda posterior no confunde un atributo comentado, o el
 * nombre de un lint citado en un `reason = "..."`, con codigo de verdad, y los
 * numeros de linea siguen valiendo.
 *
 * No es un lexer completo de Rust: cubre lo que hace falta para buscar
 * atributos (comentarios anidados, cadenas con escapes, cadenas crudas y la
 * diferencia entre un caracter y un lifetime).
 */
export function blankRustNoise(text) {
  const ranges = [];
  let index = 0;
  while (index < text.length) {
    const token = scanToken(text, index);
    if (token.blank !== undefined) ranges.push(token.blank);
    index = token.end;
  }
  return applyBlanks(text, ranges);
}

function applyBlanks(text, ranges) {
  let result = '';
  let cursor = 0;
  for (const [from, to] of ranges) {
    result += text.slice(cursor, from) + text.slice(from, to).replace(/[^\n]/gu, ' ');
    cursor = to;
  }
  return result + text.slice(cursor);
}

function scanToken(text, index) {
  const pair = text.slice(index, index + 2);
  if (pair === '//') return lineComment(text, index);
  if (pair === '/*') return blockComment(text, index);
  if (isRawStringStart(text, index)) return rawString(text, index);
  if (text[index] === '"') return string(text, index);
  if (text[index] === "'") return charLiteral(text, index);
  return { end: index + 1 };
}

function isIdentChar(char) {
  return char !== undefined && /\w/u.test(char);
}

// La r de r"..." o de br"..." abre un literal; la de un identificador (bar"), no.
function isRawStringStart(text, index) {
  if (text[index] !== 'r' || !['"', '#'].includes(text[index + 1] ?? '')) return false;
  const previous = text[index - 1];
  if (!isIdentChar(previous)) return true;
  return (previous === 'b' || previous === 'c') && !isIdentChar(text[index - 2]);
}

function lineComment(text, start) {
  const newline = text.indexOf('\n', start);
  const end = newline < 0 ? text.length : newline;
  return { end, blank: [start, end] };
}

function blockComment(text, start) {
  let depth = 0;
  let index = start;
  while (index < text.length) {
    const pair = text.slice(index, index + 2);
    if (pair === '/*') depth += 1;
    if (pair === '*/') depth -= 1;
    index += pair === '/*' || pair === '*/' ? 2 : 1;
    if (depth === 0) break;
  }
  return { end: index, blank: [start, index] };
}

function string(text, start) {
  let index = start + 1;
  while (index < text.length && text[index] !== '"') {
    index += text[index] === '\\' ? 2 : 1;
  }
  return { end: index + 1, blank: [start + 1, index] };
}

function rawString(text, start) {
  let index = start + 1;
  while (text[index] === '#') index += 1;
  if (text[index] !== '"') return { end: start + 1 };
  const closing = `"${'#'.repeat(index - start - 1)}`;
  const close = text.indexOf(closing, index + 1);
  const end = close < 0 ? text.length : close;
  return { end: end + closing.length, blank: [index + 1, end] };
}

// 'a' y '\n' son caracteres; 'a sin cierre es un lifetime y se deja tal cual.
function charLiteral(text, start) {
  const escaped = text[start + 1] === '\\';
  const close = escaped ? text.indexOf("'", start + 3) : start + 2;
  if (close < 0 || text[close] !== "'" || close - start > 12) return { end: start + 1 };
  return { end: close + 1, blank: [start + 1, close] };
}

/**
 * Atributos (`#[...]` y `#![...]`) de un fuente ya limpio, con su contenido
 * completo aunque ocupe varias lineas y la linea donde empiezan.
 */
export function rustAttributes(cleanText) {
  const found = [];
  const opener = /#!?\[/gu;
  for (let match = opener.exec(cleanText); match !== null; match = opener.exec(cleanText)) {
    const contentStart = match.index + match[0].length;
    const contentEnd = closingBracket(cleanText, contentStart);
    found.push({
      content: cleanText.slice(contentStart, contentEnd),
      line: lineOf(cleanText, match.index),
    });
  }
  return found;
}

function closingBracket(text, start) {
  let depth = 1;
  for (let index = start; index < text.length; index += 1) {
    if (text[index] === '[') depth += 1;
    if (text[index] === ']') depth -= 1;
    if (depth === 0) return index;
  }
  return text.length;
}

export function lineOf(text, offset) {
  let line = 1;
  for (let index = 0; index < offset; index += 1) {
    if (text[index] === '\n') line += 1;
  }
  return line;
}
