import { type ReactElement, useEffect } from 'react';

import type { ReleaseLoad } from '../../application/use-cases/load-release';
import type { Language } from '../../domain/preferences/preferences';
import { useUiPorts } from '../ports/ui-ports';
import { useStoredPreferences, useVisitorSnapshot } from '../preferences/page-settings';
import { SiteFooter } from '../shell/site-header';

import { DownloadsSection } from './downloads-section';
import { Hero } from './hero';
import { CtaSection, FeaturesSection, VscodeSection } from './sections';
import { useDownloadActions } from './use-download-actions';

export interface LandingScreenProps {
  /** El idioma de la URL: `/` es castellano y `/en/` ingles. */
  readonly language: Language;
  readonly release: ReleaseLoad;
}

/**
 * Leer la landing en un idioma es elegirlo: la preferencia se guarda para que
 * entrar o crear cuenta (rutas SPA, sin idioma en la URL) sigan en el mismo.
 */
function useRememberLanguage(language: Language): void {
  const { preferences } = useUiPorts();
  const stored = useStoredPreferences(preferences);
  useEffect(() => {
    if (stored.language !== language) preferences.save({ language });
  }, [language, preferences, stored.language]);
}

export function LandingScreen({ language, release }: LandingScreenProps): ReactElement {
  const { visitor } = useUiPorts();
  const { platform } = useVisitorSnapshot(visitor);
  const actions = useDownloadActions();
  const info = release.outcome === 'loaded' ? release.release : null;
  useRememberLanguage(language);

  return (
    <>
      <main>
        <Hero release={info} platform={platform} onDownload={actions.download} />
        <FeaturesSection />
        <VscodeSection onCopy={actions.copy} />
        {info === null ? null : (
          <DownloadsSection release={info} platform={platform} actions={actions} />
        )}
        <CtaSection />
      </main>
      <SiteFooter />
    </>
  );
}
