import type { BridgeClient, GraphRow } from '@git-rudder/bridge';
import { type Density, useT } from '@git-rudder/ui';
import type { ReactElement } from 'react';

import { engineErrorKey } from '../data/engine-error';
import { useGraphWindow } from '../data/use-graph-window';

import { GraphGrid } from './graph-grid';
import { GraphEmpty, GraphError, GraphLoading } from './graph-states';

export interface GraphViewProps {
  /** Cliente del motor; el host lo crea con su transporte (Tauri o VS Code). */
  readonly client: BridgeClient;
  /** Repositorio ya abierto por el host (`repo/open` es un metodo del host). */
  readonly repoId: string;
  /** comfortable en escritorio, compact en VS Code (architecture.md 7.4). */
  readonly density?: Density;
  /** Filas por peticion a `graph/window`. */
  readonly pageSize?: number;
  /** Sha seleccionado si el host controla la seleccion; sin el, la lleva la vista. */
  readonly selectedSha?: string | null;
  /** Se llama al seleccionar un commit con el raton o el teclado. */
  readonly onSelectionChange?: (row: GraphRow) => void;
  /** Doble clic o Intro sobre un commit. */
  readonly onActivate?: (row: GraphRow) => void;
}

/**
 * El grafo de commits de un repositorio: la misma vista en escritorio y en VS
 * Code, que solo cambian el cliente del motor, la densidad, el tema y los iconos.
 * Necesita por encima I18nProvider e IconProvider de @git-rudder/ui y RepoViewProvider.
 */
export function GraphView({
  client,
  repoId,
  density = 'comfortable',
  pageSize,
  ...selection
}: GraphViewProps): ReactElement {
  const t = useT();
  const graph = useGraphWindow(client, repoId, pageSize === undefined ? {} : { pageSize });
  if (graph.status === 'pending') return <GraphLoading density={density} />;
  if (graph.status === 'error') {
    return (
      <GraphError
        density={density}
        message={t(engineErrorKey(graph.error))}
        onRetry={graph.retry}
      />
    );
  }
  if (graph.rows.length === 0) return <GraphEmpty density={density} />;
  return <GraphGrid graph={graph} density={density} {...selection} />;
}
