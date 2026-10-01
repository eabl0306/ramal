import { Icon, IconButton, useT } from '@ramal/ui';
import type { ReactElement } from 'react';

import styles from './title-bar.module.css';

export interface WindowTabProps {
  readonly label: string;
  readonly active: boolean;
  readonly onActivate: () => void;
  /** Ruta completa, para el titulo al pasar el raton. */
  readonly title?: string;
  /** Sin el, la pestana no se puede cerrar (Inicio). */
  readonly onClose?: () => void;
}

/**
 * Una pestana de la barra de titulo: un boton con `aria-current` y, si se puede
 * cerrar, su boton de cerrar al lado. Las de repositorio llevan el icono de repo
 * donde el diseno pone la insignia de la plataforma (llega con las cuentas).
 */
export function WindowTab({
  label,
  active,
  onActivate,
  title,
  onClose,
}: WindowTabProps): ReactElement {
  const t = useT();
  return (
    <div className={styles['tab']} data-active={active}>
      <button
        type="button"
        className={styles['tab-button']}
        title={title}
        aria-current={active ? 'page' : undefined}
        onClick={onActivate}
      >
        {onClose === undefined ? null : <Icon name="repo" size={14} />}
        <span className={styles['tab-label']}>{label}</span>
      </button>
      {onClose === undefined ? null : (
        <IconButton icon="close" size={22} label={t('common.close')} onPress={onClose} />
      )}
    </div>
  );
}
