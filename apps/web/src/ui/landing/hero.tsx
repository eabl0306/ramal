import { type DetectedPlatform, highlightedDownloadOs } from '@git-rudder/domain';
import { Button, useT } from '@git-rudder/ui';
import type { ReactElement } from 'react';

import { recommendedFile } from '../../domain/downloads/recommend';
import { type DownloadFile, type ReleaseInfo, shortVersion } from '../../domain/downloads/release';
import { moduleClasses } from '../shared/module-classes';

import { TARGET_NAMES } from './download-labels';
import styles from './landing.module.css';

const css = moduleClasses(styles);

export interface HeroProps {
  /** null si no hubo datos de descarga al construir: la portada se pinta igual. */
  readonly release: ReleaseInfo | null;
  readonly platform: DetectedPlatform;
  readonly onDownload: (file: DownloadFile) => void;
}

/**
 * Portada. "Descargar para macOS" solo aparece cuando se sabe el sistema; antes
 * de hidratar, en un movil o en un sistema raro, el boton principal es "Todas las
 * plataformas" (el prototipo daba macOS por defecto, gaps B-55).
 */
export function Hero({ release, platform, onDownload }: HeroProps): ReactElement {
  const t = useT();
  const os = highlightedDownloadOs(platform);
  const recommended = release === null || os === null ? null : recommendedFile(release, platform);

  return (
    <section className={css('hero')}>
      <span className={css('hero-tag')}>
        {release === null ? null : (
          <span className={css('mono')}>{shortVersion(release.version)}</span>
        )}
        {t('web.hero.tag')}
      </span>
      <h1 className={css('hero-title')}>{t('web.hero.title')}</h1>
      <p className={css('hero-subtitle')}>{t('web.hero.subtitle')}</p>
      <div className={css('actions')}>
        {recommended === null || os === null ? null : (
          <Button
            variant="primary"
            size={48}
            icon="download"
            onPress={() => {
              onDownload(recommended);
            }}
          >
            {t('web.hero.downloadFor', { platform: t(TARGET_NAMES[os]) })}
          </Button>
        )}
        <a
          className={css('link-button')}
          data-variant={recommended === null ? 'primary' : undefined}
          href="#download"
        >
          {t('web.hero.allPlatforms')}
        </a>
      </div>
      <a href="#vscode">{t('web.hero.vscodeLink')}</a>
    </section>
  );
}
