import { LOCALES, type Locale } from '@ramal/i18n';
import { type SegmentOption, SegmentedControl, useT } from '@ramal/ui';
import type { ReactElement } from 'react';

import type { ThemeName } from '../../application/ports/appearance-store';
import { useAppearance } from '../appearance/appearance-context';

import styles from './title-bar.module.css';

/**
 * Los dos conmutadores de la derecha de la barra de titulo: idioma (ES/EN) y tema
 * (claro/oscuro). Las siglas del idioma son codigos, no textos que traducir; el
 * nombre accesible de cada grupo si sale de @ramal/i18n.
 */
export function AppearanceSwitches(): ReactElement {
  const t = useT();
  const { theme, locale, setTheme, setLocale } = useAppearance();
  const languages: readonly SegmentOption<Locale>[] = LOCALES.map((code) => ({
    id: code,
    label: code.toUpperCase(),
  }));
  const themes: readonly SegmentOption<ThemeName>[] = [
    { id: 'light', label: t('common.theme.light'), icon: 'theme-light', iconOnly: true },
    { id: 'dark', label: t('common.theme.dark'), icon: 'theme-dark', iconOnly: true },
  ];
  return (
    <div className={styles['switches']}>
      <SegmentedControl
        variant="bar"
        label={t('settings.appearance.language')}
        options={languages}
        value={locale}
        onChange={setLocale}
      />
      <SegmentedControl
        variant="bar"
        label={t('settings.appearance.theme')}
        options={themes}
        value={theme}
        onChange={setTheme}
      />
    </div>
  );
}
