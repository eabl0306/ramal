import { Button, type Density, useT } from '@ramal/ui';
import type { ReactElement } from 'react';

import { moduleClasses } from './css';
import styles from './graph-states.module.css';

const css = moduleClasses(styles);

interface StateProps {
  readonly density: Density;
}

/** Primera carga: todavia no hay ninguna fila que ensenar. */
export function GraphLoading({ density }: StateProps): ReactElement {
  const t = useT();
  return (
    <div className={css('state')} data-density={density} role="status" aria-busy="true">
      <p className={css('detail')}>{t('common.loading')}</p>
    </div>
  );
}

/** Repositorio sin commits (gaps D-10 y V-09): una rama sin nacer. */
export function GraphEmpty({ density }: StateProps): ReactElement {
  const t = useT();
  return (
    <div className={css('state')} data-density={density} role="status">
      <p className={css('title')}>{t('graph.view.empty')}</p>
    </div>
  );
}

export interface GraphErrorProps extends StateProps {
  /** Texto traducido del `kind` del error. */
  readonly message: string;
  readonly onRetry: () => void;
}

/** La primera pagina fallo: que paso, en el idioma de la interfaz, y reintentar. */
export function GraphError({ density, message, onRetry }: GraphErrorProps): ReactElement {
  const t = useT();
  return (
    <div className={css('state')} data-density={density} role="alert">
      <p className={css('title')}>{t('graph.view.error')}</p>
      <p className={css('detail')}>{message}</p>
      <Button size={30} onPress={onRetry}>
        {t('graph.view.retry')}
      </Button>
    </div>
  );
}
