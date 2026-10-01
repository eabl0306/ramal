/**
 * Las dos densidades en numeros. El CSS (themes/base.css) tiene los tokens de
 * tamano, pero el SVG del grafo necesita los valores en JavaScript: alto de fila,
 * origen X, radio de nodo y puntos de control de las curvas. Un test parsea el
 * CSS y falla si alguno de los cuatro tokens no coincide con esto.
 *
 * Comfortable es escritorio y web; compact es VS Code. Los numeros del grafo son
 * los de los prototipos (data-and-algorithms, seccion 3.7).
 */

export const DENSITY_NAMES = ['comfortable', 'compact'] as const;

export type Density = (typeof DENSITY_NAMES)[number];

export interface GraphGeometry {
  readonly laneWidth: number;
  /** X del carril 0 dentro del SVG. */
  readonly originX: number;
  /** Radio del nodo de un commit normal con iniciales. */
  readonly nodeRadius: number;
  /** Radio del nodo en modo punto. */
  readonly dotRadius: number;
  /** Radio del nodo de un merge (dos padres o mas). */
  readonly mergeRadius: number;
  readonly wipRadius: number;
  /** Lo que crece el anillo del commit seleccionado sobre el radio del nodo. */
  readonly selectedRingOffset: number;
  /** Puntos de control de las curvas, en px desde el extremo de la fila. */
  readonly controlNear: number;
  readonly controlFar: number;
}

export interface DensityMetrics {
  /** --r-row-h: fila del grafo. */
  readonly rowHeight: number;
  /** --r-menu-item-h */
  readonly menuItemHeight: number;
  /** --r-sidebar-item-h */
  readonly sidebarItemHeight: number;
  /** --r-toolbar-h */
  readonly toolbarHeight: number;
  readonly graph: GraphGeometry;
}

export const DENSITIES: Readonly<Record<Density, DensityMetrics>> = {
  comfortable: {
    rowHeight: 40,
    menuItemHeight: 32,
    sidebarItemHeight: 30,
    toolbarHeight: 62,
    graph: {
      laneWidth: 20,
      originX: 16,
      nodeRadius: 9,
      dotRadius: 6,
      mergeRadius: 5,
      wipRadius: 8,
      selectedRingOffset: 4,
      controlNear: 22,
      controlFar: 18,
    },
  },
  compact: {
    rowHeight: 28,
    menuItemHeight: 24,
    sidebarItemHeight: 22,
    toolbarHeight: 34,
    graph: {
      laneWidth: 20,
      originX: 14,
      nodeRadius: 7,
      dotRadius: 7,
      mergeRadius: 4,
      wipRadius: 6,
      selectedRingOffset: 3,
      controlNear: 16,
      controlFar: 12,
    },
  },
};

/** El token CSS de cada metrica, en el orden en que los compara el test. */
export const DENSITY_TOKENS: Readonly<Record<Exclude<keyof DensityMetrics, 'graph'>, string>> = {
  rowHeight: '--r-row-h',
  menuItemHeight: '--r-menu-item-h',
  sidebarItemHeight: '--r-sidebar-item-h',
  toolbarHeight: '--r-toolbar-h',
};
