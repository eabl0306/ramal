// API publica de @ramal/repo-view: lo que montan escritorio y los webviews de
// VS Code. El resto (geometria, caminos, columnas) es interno y puede cambiar.

export { GraphView, type GraphViewProps } from './view/graph-view';
export { RepoViewProvider, type RepoViewProviderProps } from './view/repo-view-provider';
export {
  type GraphWindowState,
  type UseGraphWindowOptions,
  useGraphWindow,
} from './data/use-graph-window';
export { DEFAULT_PAGE_SIZE, type GraphWindowKey, graphWindowKey } from './data/graph-window-query';
export { createRepoViewQueryClient } from './data/query-client';
export { ENGINE_ERROR_KEYS, engineErrorKey } from './data/engine-error';
