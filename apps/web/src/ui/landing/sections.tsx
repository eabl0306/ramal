import type { PlainKey } from '@ramal/i18n';
import { useT } from '@ramal/ui';
import type { ReactElement } from 'react';
import { Link } from 'react-router';

import { INSTALL_COMMANDS, STORE_URLS } from '../../domain/downloads/release';
import { moduleClasses } from '../shared/module-classes';

import { CommandBox } from './command-box';
import styles from './landing.module.css';

const css = moduleClasses(styles);

/** Antetitulo monoespaciado y titulo de seccion; reciben claves, no textos. */
export function SectionHeading({
  kickerKey,
  titleKey,
}: {
  readonly kickerKey: PlainKey;
  readonly titleKey: PlainKey;
}): ReactElement {
  const t = useT();
  return (
    <div className={css('heading')}>
      <div className={css('kicker')}>{t(kickerKey)}</div>
      <h2 className={css('section-title')}>{t(titleKey)}</h2>
    </div>
  );
}

/** Las seis funciones del diseno, numeradas del 01 al 06. */
const FEATURES = ['graph', 'conflicts', 'accounts', 'workspaces', 'ai', 'cloud'] as const;

export function FeaturesSection(): ReactElement {
  const t = useT();
  return (
    <section id="features" className={css('section')}>
      <SectionHeading kickerKey="web.features.kicker" titleKey="web.features.title" />
      <div className={css('features')}>
        {FEATURES.map((feature, index) => (
          <article key={feature} className={css('feature')}>
            <span className={css('feature-number')}>{String(index + 1).padStart(2, '0')}</span>
            <h3 className={css('feature-title')}>{t(`web.features.items.${feature}.title`)}</h3>
            <p className={css('feature-text')}>{t(`web.features.items.${feature}.text`)}</p>
          </article>
        ))}
      </div>
    </section>
  );
}

/** Marketplace y Open VSX son enlaces fijos del codigo (ADR 0016), no datos de una publicacion. */
export function VscodeSection({
  onCopy,
}: {
  readonly onCopy: (command: string) => void;
}): ReactElement {
  const t = useT();
  return (
    <section id="vscode" className={`${css('section')} ${css('vscode')}`}>
      <SectionHeading kickerKey="web.vscode.kicker" titleKey="web.vscode.title" />
      <p className={css('lead')}>{t('web.vscode.text')}</p>
      <div className={css('row')}>
        <a
          className={css('link-button')}
          data-variant="primary"
          data-size="42"
          href={STORE_URLS.marketplace}
          target="_blank"
          rel="noreferrer"
        >
          {t('web.vscode.marketplace')}
        </a>
        <a
          className={css('link-button')}
          data-size="42"
          href={STORE_URLS['open-vsx']}
          target="_blank"
          rel="noreferrer"
        >
          {t('web.vscode.openVsx')}
        </a>
      </div>
      <CommandBox command={INSTALL_COMMANDS.vscode} onCopy={onCopy} />
    </section>
  );
}

export function CtaSection(): ReactElement {
  const t = useT();
  return (
    <section className={css('section')}>
      <div className={css('cta')}>
        <div className={css('cta-body')}>
          <h2 className={css('cta-title')}>{t('web.cta.title')}</h2>
          <p className={css('feature-text')}>{t('web.cta.text')}</p>
        </div>
        <Link to="/signup" className={css('link-button')} data-variant="primary">
          {t('web.nav.signup')}
        </Link>
      </div>
    </section>
  );
}
