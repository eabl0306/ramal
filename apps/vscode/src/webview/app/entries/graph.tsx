import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';

import { connectToHost } from '../../infrastructure/host-connection';
import { vsCodeApi } from '../../infrastructure/vscode-api';
import { GraphApp } from '../../ui/graph-app';

/**
 * Entrada del webview `ramal.graph`. Pide el `init` al host y monta el grafo con
 * el idioma y el repositorio que este le da; hasta entonces no pinta nada.
 */
const root = document.getElementById('root');
if (root === null) throw new Error('the webview HTML has no #root element');

const host = connectToHost(vsCodeApi(), window, 'graph');

void host.init.then((init) => {
  createRoot(root).render(
    <StrictMode>
      <GraphApp client={host.client} locale={init.locale} repoId={init.repoId} />
    </StrictMode>,
  );
});
