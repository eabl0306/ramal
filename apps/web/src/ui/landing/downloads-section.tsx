import { type DetectedPlatform, highlightedDownloadOs } from '@ramal/domain';
import { formatDate } from '@ramal/i18n';
import { useLocale, useT } from '@ramal/ui';
import type { ReactElement } from 'react';

import { isMobileVisitor } from '../../domain/downloads/recommend';
import { type ReleaseInfo, releaseDay } from '../../domain/downloads/release';
import { moduleClasses } from '../shared/module-classes';

import { DownloadCardView } from './download-card';
import styles from './downloads.module.css';
import landing from './landing.module.css';
import { SectionHeading } from './sections';
import type { DownloadActions } from './use-download-actions';

const css = moduleClasses(styles);
const section = moduleClasses(landing);

export interface DownloadsSectionProps {
  readonly release: ReleaseInfo;
  readonly platform: DetectedPlatform;
  readonly actions: DownloadActions;
}

/**
 * Una tarjeta por sistema mas la de VS Code. La del sistema de quien visita se
 * destaca al hidratar, porque detectar el sistema al construir no tiene sentido
 * (ADR 0015); en un movil no se destaca ninguna y se avisa de que Ramal es de
 * escritorio (gaps B-55).
 */
export function DownloadsSection({
  release,
  platform,
  actions,
}: DownloadsSectionProps): ReactElement {
  const t = useT();
  const locale = useLocale();
  const highlighted = highlightedDownloadOs(platform);
  const date = formatDate(releaseDay(release.releasedAt), locale, { timeZone: 'UTC' });

  return (
    <section id="download" className={section('section')}>
      <div className={css('head')}>
        <SectionHeading kickerKey="web.download.kicker" titleKey="web.download.title" />
        {/* El ICU de Node y el del navegador pueden escribir la fecha distinto. */}
        <span className={css('version')} suppressHydrationWarning>
          {t('web.download.version', { version: release.version, date })}
        </span>
      </div>
      {isMobileVisitor(platform) ? (
        <p className={css('notice')} role="note">
          {t('web.download.mobile')}
        </p>
      ) : null}
      <div className={css('grid')}>
        {release.cards.map((card) => (
          <DownloadCardView
            key={card.target}
            card={card}
            highlighted={card.target === highlighted}
            actions={actions}
          />
        ))}
      </div>
    </section>
  );
}
