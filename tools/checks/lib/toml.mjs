/**
 * Lectura minima de TOML para los scripts de tools/checks, que no tienen
 * dependencias. No es un parser completo: entiende tablas (`[a.b]`), claves con
 * valor en una linea y arrays que ocupan varias lineas, que es lo que usan
 * Cargo.toml y clippy.toml en este repo. Devuelve los valores como texto crudo.
 */
export function parseTomlLite(text) {
  const reader = new TomlReader();
  for (const rawLine of text.split('\n')) reader.read(stripComment(rawLine).trim());
  return reader.entries;
}

class TomlReader {
  entries = [];
  table = '';
  pending = null;

  // Una linea cierra el array pendiente, abre una tabla o es una clave = valor.
  read(line) {
    if (this.pending !== null) {
      this.continuePending(line);
      return;
    }
    const header = /^\[{1,2}\s*([^\]]+?)\s*\]{1,2}$/u.exec(line);
    if (header !== null) {
      this.table = header[1] ?? '';
      return;
    }
    const pair = /^([\w.-]+|"[^"]+")\s*=\s*(.*)$/u.exec(line);
    if (pair === null) return;
    const entry = { table: this.table, key: unquote(pair[1] ?? ''), value: pair[2] ?? '' };
    if (bracketBalance(entry.value) > 0) this.pending = entry;
    else this.entries.push(entry);
  }

  continuePending(line) {
    const entry = { ...this.pending, value: `${this.pending.value} ${line}` };
    if (bracketBalance(entry.value) > 0) {
      this.pending = entry;
      return;
    }
    this.entries.push(entry);
    this.pending = null;
  }
}

/** Valor de una clave en una tabla (la raiz es ''), o undefined. */
export function tomlValue(entries, table, key) {
  return entries.find((entry) => entry.table === table && entry.key === key)?.value;
}

/** Las cadenas de un array TOML (`["a", "b"]`), en orden. */
export function tomlStrings(value) {
  return [...(value ?? '').matchAll(/"([^"]*)"/gu)].map((match) => match[1] ?? '');
}

function unquote(key) {
  return key.replace(/^"(.*)"$/u, '$1');
}

// Un # dentro de una cadena no abre comentario.
function stripComment(line) {
  let quote = null;
  for (let index = 0; index < line.length; index += 1) {
    const char = line[index];
    if (quote !== null) {
      if (char === '\\') index += 1;
      else if (char === quote) quote = null;
    } else if (char === '"' || char === "'") {
      quote = char;
    } else if (char === '#') {
      return line.slice(0, index);
    }
  }
  return line;
}

function bracketBalance(value) {
  let balance = 0;
  for (const char of value.replace(/"(?:[^"\\]|\\.)*"/gu, '""')) {
    if (char === '[' || char === '{') balance += 1;
    if (char === ']' || char === '}') balance -= 1;
  }
  return balance;
}
