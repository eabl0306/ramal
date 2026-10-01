import { render, screen } from '@testing-library/react';
import type { ReactElement } from 'react';
import { describe, expect, it, vi } from 'vitest';

import { useLocale, useT } from './i18n-context';
import { I18nProvider } from './i18n-provider';

function Probe(): ReactElement {
  const t = useT();
  const locale = useLocale();
  return (
    <p>
      {locale}: {t('toast.pushed', { branch: 'main' })}
    </p>
  );
}

describe('I18nProvider y useT', () => {
  it('traduce con el idioma del provider', () => {
    render(
      <I18nProvider locale="es">
        <Probe />
      </I18nProvider>,
    );
    expect(screen.getByText('es: Push a main completado')).toBeInTheDocument();
  });

  it('cambia de idioma al cambiar el provider', () => {
    const { rerender } = render(
      <I18nProvider locale="es">
        <Probe />
      </I18nProvider>,
    );
    rerender(
      <I18nProvider locale="en">
        <Probe />
      </I18nProvider>,
    );
    expect(screen.getByText('en: Pushed to main')).toBeInTheDocument();
  });

  it('sin provider falla con un mensaje claro', () => {
    vi.spyOn(console, 'error').mockImplementation(() => undefined);
    expect(() => render(<Probe />)).toThrow(/I18nProvider/u);
  });
});
