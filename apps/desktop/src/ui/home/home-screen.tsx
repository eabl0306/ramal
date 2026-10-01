import type { BridgeErrorKind } from '@ramal/bridge';
import { ENGINE_ERROR_KEYS } from '@ramal/repo-view';
import { Button, useT } from '@ramal/ui';
import type { ReactElement } from 'react';

import styles from './home-screen.module.css';

export interface HomeScreenProps {
  /** Se espera al dialogo de carpetas: el boton no admite otra pulsacion. */
  readonly picking: boolean;
  /** Por que fallo la ultima apertura, ya como `kind` del motor. */
  readonly error: BridgeErrorKind | null;
  readonly onOpen: () => void;
}

/**
 * Inicio en el primer corte: abrir un repositorio local. La lista de repositorios,
 * los workspaces y el primer arranque del diseno estan planificados (gaps D-01 y
 * D-05); aqui solo esta la entrada que lleva al grafo.
 */
export function HomeScreen({ picking, error, onOpen }: HomeScreenProps): ReactElement {
  const t = useT();
  return (
    <section className={styles['home']} aria-labelledby="home-title">
      <h1 id="home-title" className={styles['title']}>
        {t('repos.title')}
      </h1>
      <div className={styles['actions']}>
        <Button variant="primary" icon="folder" isDisabled={picking} onPress={onOpen}>
          {t('dialog.open.title')}
        </Button>
        {picking ? (
          <span className={styles['status']} role="status">
            {t('common.loading')}
          </span>
        ) : null}
      </div>
      {error === null ? null : (
        <p className={styles['error']} role="alert">
          {t(ENGINE_ERROR_KEYS[error])}
        </p>
      )}
    </section>
  );
}
