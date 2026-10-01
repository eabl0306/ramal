import '@ramal/ui/themes/vscode.css';

import type { BridgeClient } from '@ramal/bridge';
import type { Locale } from '@ramal/i18n';
import { GraphView, RepoViewProvider } from '@ramal/repo-view';
import { I18nProvider, IconProvider } from '@ramal/ui';
import { codicons } from '@ramal/ui/icons/codicons';
import type { ReactElement } from 'react';

import styles from './graph-app.module.css';

export interface GraphAppProps {
  readonly client: BridgeClient;
  readonly locale: Locale;
  readonly repoId: string;
}

/**
 * La superficie del grafo en VS Code: el mismo GraphView que escritorio, con el
 * tema que sigue al editor (vscode.css), densidad compacta y Codicons. Las
 * diferencias con escritorio son tokens, densidad e iconos, no ramas en el
 * componente (architecture.md 7.4).
 */
export function GraphApp({ client, locale, repoId }: GraphAppProps): ReactElement {
  return (
    <I18nProvider locale={locale}>
      <IconProvider icons={codicons}>
        <RepoViewProvider>
          <main className={styles['screen']} data-density="compact">
            <GraphView client={client} repoId={repoId} density="compact" />
          </main>
        </RepoViewProvider>
      </IconProvider>
    </I18nProvider>
  );
}
