import { Buffer } from 'node:buffer';

/**
 * Marcos `Content-Length` del sidecar, como LSP (architecture.md 6.5):
 * `Content-Length: <bytes>\r\n\r\n` y exactamente esos bytes de JSON en UTF-8.
 * Las demas cabeceras se ignoran. Un marco de mas de 32 MiB se rechaza y quien
 * lee cierra la conexion, igual que hace el sidecar en su lado.
 */
export const MAX_FRAME_BYTES = 32 * 1024 * 1024;

/** Tope del bloque de cabeceras: una cabecera sin fin no puede llenar la memoria. */
export const MAX_HEADER_BYTES = 64 * 1024;

const HEADER_END = Buffer.from('\r\n\r\n', 'latin1');
const DIGITS = /^\d+$/u;

/** El flujo del motor no se puede leer; la conexion se cierra. */
export class FrameError extends Error {
  override readonly name = 'FrameError';
}

/** El marco de un cuerpo JSON. La longitud es en bytes, no en caracteres. */
export function encodeFrame(body: string): Buffer {
  const bytes = Buffer.from(body, 'utf8');
  const header = Buffer.from(`Content-Length: ${String(bytes.length)}\r\n\r\n`, 'latin1');
  return Buffer.concat([header, bytes]);
}

/** Lee `Content-Length` de un bloque de cabeceras sin la linea vacia final. */
export function parseContentLength(headers: string): number {
  let length: number | undefined;
  for (const line of headers.split('\r\n')) {
    const separator = line.indexOf(':');
    if (separator === -1) continue;
    if (line.slice(0, separator).trim().toLowerCase() !== 'content-length') continue;
    const value = line.slice(separator + 1).trim();
    if (!DIGITS.test(value)) throw new FrameError(`invalid Content-Length: ${value}`);
    length = Number(value);
  }
  if (length === undefined) throw new FrameError('frame headers have no Content-Length');
  if (length > MAX_FRAME_BYTES) {
    throw new FrameError(`frame of ${String(length)} bytes exceeds the limit`);
  }
  return length;
}

/**
 * Junta los trozos que llegan por stdout y saca los cuerpos completos. Guarda
 * los trozos sin copiarlos mientras espera un cuerpo grande y solo los junta
 * cuando ya estan todos sus bytes: cada marco se copia una vez.
 */
export class FrameDecoder {
  #chunks: Buffer[] = [];
  #length = 0;
  #expected: number | null = null;

  /** Anade bytes y devuelve los cuerpos completos. Lanza `FrameError` si el flujo esta roto. */
  push(chunk: Buffer): string[] {
    this.#chunks.push(chunk);
    this.#length += chunk.length;
    const bodies: string[] = [];
    for (let body = this.#next(); body !== null; body = this.#next()) bodies.push(body);
    return bodies;
  }

  #next(): string | null {
    if (this.#expected === null && !this.#readHeaders()) return null;
    const expected = this.#expected ?? 0;
    if (this.#length < expected) return null;
    const all = this.#flatten();
    const body = all.subarray(0, expected).toString('utf8');
    this.#keep(all.subarray(expected));
    this.#expected = null;
    return body;
  }

  #readHeaders(): boolean {
    if (this.#length === 0) return false;
    const all = this.#flatten();
    const end = all.indexOf(HEADER_END);
    if (end === -1) {
      if (all.length > MAX_HEADER_BYTES) throw new FrameError('frame headers are too long');
      return false;
    }
    this.#expected = parseContentLength(all.subarray(0, end).toString('latin1'));
    this.#keep(all.subarray(end + HEADER_END.length));
    return true;
  }

  #flatten(): Buffer {
    const all = this.#chunks.length === 1 ? this.#chunks[0] : Buffer.concat(this.#chunks);
    return all ?? Buffer.alloc(0);
  }

  #keep(rest: Buffer): void {
    this.#chunks = rest.length === 0 ? [] : [rest];
    this.#length = rest.length;
  }
}
