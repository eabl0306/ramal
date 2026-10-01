import { type ReactElement, useCallback, useReducer } from 'react';

import {
  INITIAL_WORKSPACE,
  activeTab,
  workspaceReducer,
} from '../../application/workspace/workspace';
import { HomeScreen } from '../home/home-screen';
import { useDesktopHost } from '../host/desktop-host-context';
import { RepoScreen } from '../repo/repo-screen';

import styles from './app-shell.module.css';
import { TitleBar } from './title-bar';

/**
 * La ventana: barra de titulo con pestanas y, debajo, Inicio o el repositorio de
 * la pestana visible. El estado de las pestanas es el reducer de la capa de
 * aplicacion; aqui solo se cablea con el selector del host.
 */
export function AppShell(): ReactElement {
  const { client, picker } = useDesktopHost();
  const [state, dispatch] = useReducer(workspaceReducer, INITIAL_WORKSPACE);

  const openRepository = useCallback(async () => {
    dispatch({ type: 'pick-started' });
    dispatch({ type: 'pick-finished', result: await picker.pickAndOpen() });
  }, [picker]);

  const current = activeTab(state);
  return (
    <div className={styles['shell']}>
      <TitleBar
        tabs={state.tabs}
        active={state.active}
        onActivate={(repoId) => {
          dispatch({ type: 'tab-activated', repoId });
        }}
        onClose={(repoId) => {
          dispatch({ type: 'tab-closed', repoId });
        }}
      />
      <main className={styles['content']}>
        {current === null ? (
          <HomeScreen
            picking={state.picking}
            error={state.openError}
            onOpen={() => {
              void openRepository();
            }}
          />
        ) : (
          <RepoScreen key={current.repoId} client={client} tab={current} />
        )}
      </main>
    </div>
  );
}
