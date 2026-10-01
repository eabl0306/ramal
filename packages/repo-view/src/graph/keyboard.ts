/**
 * Teclado de la tabla del grafo: a que fila lleva cada tecla. Es una funcion
 * pura para probar los bordes sin montar nada; el componente solo la llama y
 * mueve la seleccion.
 *
 * Sin seleccion, bajar o Inicio van a la primera fila y subir o Fin a la ultima,
 * que es lo que hace una lista de VS Code.
 */
type Move = (current: number | null, last: number, page: number) => number;

/** Un Map y no un objeto: una tecla como "toString" no puede encontrar nada. */
const MOVES = new Map<string, Move>([
  ['ArrowDown', (current) => (current === null ? 0 : current + 1)],
  ['ArrowUp', (current, last) => (current === null ? last : current - 1)],
  ['PageDown', (current, _last, page) => (current ?? -1) + page],
  ['PageUp', (current, last, page) => (current ?? last + 1) - page],
  ['Home', () => 0],
  ['End', (_current, last) => last],
]);

/**
 * Fila de destino, siempre dentro de la tabla. Devuelve `null` si la tecla no es
 * de navegacion o no hay filas: entonces el evento sigue su camino.
 */
export function nextRowIndex(
  key: string,
  current: number | null,
  count: number,
  pageSize: number,
): number | null {
  const move = MOVES.get(key);
  if (move === undefined || count <= 0) return null;
  const last = count - 1;
  const target = move(current, last, Math.max(1, Math.floor(pageSize)));
  return Math.min(Math.max(target, 0), last);
}
