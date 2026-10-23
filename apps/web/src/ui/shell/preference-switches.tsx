import { type SegmentOption, SegmentedControl, useLocale, useT } from '@git-rudder/ui';
import type { ReactElement } from 'react';
import { useLocation, useNavigate } from 'react-router';

import { landingLanguage, landingPath } from '../../domain/preferences/page-preferences';
import { LANGUAGES, type Language, type Theme } from '../../domain/preferences/preferences';
import { useUiPorts } from '../ports/ui-ports';
import { usePageSettings } from '../preferences/page-settings';

/** El codigo del idioma es lo que ensena el diseno (ES, EN): no se traduce. */
const LANGUAGE_OPTIONS: readonly SegmentOption<Language>[] = LANGUAGES.map((id) => ({
  id,
  label: id.toUpperCase(),
}));

/**
 * ES/EN de la barra. En la landing el idioma va en la URL, asi que cambiarlo es
 * ir a la otra landing (con la misma seccion); en el resto de rutas basta con
 * guardar la preferencia.
 */
export function LanguageSwitch(): ReactElement {
  const t = useT();
  const locale = useLocale();
  const { preferences } = useUiPorts();
  const { pathname, hash } = useLocation();
  const navigate = useNavigate();

  const change = (language: Language): void => {
    preferences.save({ language });
    if (landingLanguage(pathname) !== null) {
      void navigate({ pathname: landingPath(language), hash });
    }
  };

  return (
    <SegmentedControl
      variant="nav"
      label={t('settings.appearance.language')}
      value={locale}
      options={LANGUAGE_OPTIONS}
      onChange={change}
    />
  );
}

/** Claro u oscuro. Lo elegido se recuerda y gana al tema del sistema. */
export function ThemeSwitch(): ReactElement {
  const t = useT();
  const { preferences, visitor } = useUiPorts();
  const { theme } = usePageSettings(preferences, visitor);
  const options: readonly SegmentOption<Theme>[] = [
    { id: 'light', label: t('common.theme.light'), icon: 'theme-light', iconOnly: true },
    { id: 'dark', label: t('common.theme.dark'), icon: 'theme-dark', iconOnly: true },
  ];

  return (
    <SegmentedControl
      variant="nav"
      label={t('settings.appearance.theme')}
      value={theme}
      options={options}
      onChange={(next) => {
        preferences.save({ theme: next });
      }}
    />
  );
}
