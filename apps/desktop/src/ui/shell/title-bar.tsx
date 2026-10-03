import { useT } from '@git-rudder/ui';
import type { ReactElement } from 'react';

import type { RepoTab } from '../../application/workspace/workspace';

import { AppearanceSwitches } from './appearance-switches';
import { PRODUCT_INITIAL, PRODUCT_NAME } from './brand';
import styles from './title-bar.module.css';
import { WindowTab } from './window-tab';

export interface TitleBarProps {
  readonly tabs: readonly RepoTab[];
  /** `repoId` de la pestana visible, o `null` en Inicio. */
  readonly active: string | null;
  readonly onActivate: (repoId: string | null) => void;
  readonly onClose: (repoId: string) => void;
}

/**
 * La barra de 46 px del diseno: logo, pestana de Inicio, una pestana por
 * repositorio abierto y, a la derecha, idioma y tema. Las pestanas son botones con
 * `aria-current`, porque cada una lleva dentro su boton de cerrar y una pestana de
 * ARIA no puede contener otro control.
 */
export function TitleBar({ tabs, active, onActivate, onClose }: TitleBarProps): ReactElement {
  const t = useT();
  return (
    <header className={styles['bar']}>
      <div className={styles['brand']}>
        <span className={styles['logo']} aria-hidden="true">
          {PRODUCT_INITIAL}
        </span>
        <span className={styles['name']}>{PRODUCT_NAME}</span>
      </div>
      <nav className={styles['tabs']}>
        <WindowTab
          label={t('common.home')}
          active={active === null}
          onActivate={() => {
            onActivate(null);
          }}
        />
        {tabs.map((tab) => (
          <WindowTab
            key={tab.repoId}
            label={tab.label}
            title={tab.root}
            active={tab.repoId === active}
            onActivate={() => {
              onActivate(tab.repoId);
            }}
            onClose={() => {
              onClose(tab.repoId);
            }}
          />
        ))}
      </nav>
      <AppearanceSwitches />
    </header>
  );
}
