/**
 * Parte en lineas el texto que llega a trozos por stderr: un trozo puede cortar
 * una linea por la mitad, y el log del motor se vuelca linea a linea.
 */
export class LineSplitter {
  #rest = '';

  push(text: string): string[] {
    const parts = `${this.#rest}${text}`.split(/\r?\n/u);
    this.#rest = parts.pop() ?? '';
    return parts;
  }

  /** Lo que quedo sin salto de linea al final, o `null`. */
  flush(): string | null {
    const rest = this.#rest;
    this.#rest = '';
    return rest === '' ? null : rest;
  }
}
