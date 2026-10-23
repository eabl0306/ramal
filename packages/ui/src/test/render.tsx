import type { Locale } from '@git-rudder/i18n';
import { type RenderResult, render } from '@testing-library/react';
import type { ReactElement } from 'react';

import { I18nProvider } from '../i18n/i18n-provider';
import { IconProvider } from '../icons/icon';
import type { IconSet } from '../icons/icon-set';
import { lucideIcons } from '../icons/lucide';

export interface RenderOptions {
  readonly locale?: Locale;
  readonly icons?: IconSet;
}

/** Monta con los mismos providers que pone cada host en su composition root. */
export function renderWithProviders(
  ui: ReactElement,
  { locale = 'es', icons = lucideIcons }: RenderOptions = {},
): RenderResult {
  return render(
    <I18nProvider locale={locale}>
      <IconProvider icons={icons}>{ui}</IconProvider>
    </I18nProvider>,
  );
}
