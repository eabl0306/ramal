import type { BridgeClient } from '@ramal/bridge';
import { GraphView } from '@ramal/repo-view';
import type { ReactElement } from 'react';

import type { RepoTab } from '../../application/workspace/workspace';

import styles from './repo-screen.module.css';

export interface RepoScreenProps {
  readonly client: BridgeClient;
  readonly tab: RepoTab;
}

/**
 * La vista de un repositorio: el grafo de @ramal/repo-view, el mismo componente
 * que en VS Code, en densidad comoda. La barra de herramientas, la barra lateral y
 * el panel derecho del diseno llegan con sus casos de uso en el motor.
 */
export function RepoScreen({ client, tab }: RepoScreenProps): ReactElement {
  return (
    <section className={styles['repo']} aria-label={tab.label}>
      <GraphView client={client} repoId={tab.repoId} density="comfortable" />
    </section>
  );
}
