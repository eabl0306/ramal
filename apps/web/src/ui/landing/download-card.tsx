import { Badge, Button, useT } from '@ramal/ui';
import type { ReactElement } from 'react';

import {
  type DownloadCard,
  type DownloadEntry,
  INSTALL_COMMANDS,
  STORE_URLS,
  sizeInMegabytes,
} from '../../domain/downloads/release';
import { moduleClasses } from '../shared/module-classes';

import { CommandBox } from './command-box';
import {
  ARTIFACT_LABELS,
  STORE_LABELS,
  TARGET_NAMES,
  TARGET_REQUIREMENTS,
} from './download-labels';
import styles from './downloads.module.css';
import type { DownloadActions } from './use-download-actions';

const css = moduleClasses(styles);

interface EntryProps {
  readonly entry: DownloadEntry;
  readonly actions: DownloadActions;
}

function PrimaryEntry({ entry, actions }: EntryProps): ReactElement {
  const t = useT();
  if (entry.kind === 'store') {
    return (
      <a
        className={css('primary-link')}
        href={STORE_URLS[entry.store]}
        target="_blank"
        rel="noreferrer"
      >
        {t(STORE_LABELS[entry.store])}
      </a>
    );
  }
  const { file } = entry;
  return (
    <Button
      variant="primary"
      size={42}
      block
      icon="download"
      onPress={() => {
        actions.download(file);
      }}
    >
      {t(ARTIFACT_LABELS[file.label])}
    </Button>
  );
}

function AlternateEntry({ entry, actions }: EntryProps): ReactElement {
  const t = useT();
  if (entry.kind === 'store') {
    return (
      <a
        className={css('alternate')}
        href={STORE_URLS[entry.store]}
        target="_blank"
        rel="noreferrer"
      >
        <span>{t(STORE_LABELS[entry.store])}</span>
      </a>
    );
  }
  const { file } = entry;
  return (
    <button
      type="button"
      className={css('alternate')}
      onClick={() => {
        actions.download(file);
      }}
    >
      <span>{t(ARTIFACT_LABELS[file.label])}</span>
      {file.sizeBytes === null ? null : (
        <span className={css('size')}>
          {t('web.download.size', { size: sizeInMegabytes(file.sizeBytes) })}
        </span>
      )}
    </button>
  );
}

function entryKey(entry: DownloadEntry): string {
  return entry.kind === 'store' ? entry.store : entry.file.id;
}

export interface DownloadCardViewProps {
  readonly card: DownloadCard;
  /** La tarjeta del sistema de quien visita lleva borde de acento y "Tu sistema". */
  readonly highlighted: boolean;
  readonly actions: DownloadActions;
}

export function DownloadCardView({
  card,
  highlighted,
  actions,
}: DownloadCardViewProps): ReactElement {
  const t = useT();
  return (
    <article
      className={css('card')}
      data-testid={`download-card-${card.target}`}
      data-highlighted={highlighted ? 'true' : 'false'}
    >
      <div className={css('card-head')}>
        <h3 className={css('card-name')}>{t(TARGET_NAMES[card.target])}</h3>
        {highlighted ? <Badge tone="accent">{t('web.download.yourSystem')}</Badge> : null}
      </div>
      <p className={css('requirement')}>{t(TARGET_REQUIREMENTS[card.target])}</p>
      <PrimaryEntry entry={card.primary} actions={actions} />
      <ul className={css('alternates')}>
        {card.alternates.map((entry) => (
          <li key={entryKey(entry)}>
            <AlternateEntry entry={entry} actions={actions} />
          </li>
        ))}
      </ul>
      <CommandBox command={INSTALL_COMMANDS[card.install]} onCopy={actions.copy} compact />
    </article>
  );
}
