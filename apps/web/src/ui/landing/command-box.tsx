import { useT } from '@ramal/ui';
import type { ReactElement } from 'react';

import { moduleClasses } from '../shared/module-classes';

import styles from './downloads.module.css';

const css = moduleClasses(styles);

export interface CommandBoxProps {
  readonly command: string;
  readonly onCopy: (command: string) => void;
  /** La version pequena de las tarjetas de descarga. */
  readonly compact?: boolean;
}

/** Un comando de instalacion que se copia con un clic. El "$" es decorativo. */
export function CommandBox({ command, onCopy, compact = false }: CommandBoxProps): ReactElement {
  const t = useT();
  return (
    <button
      type="button"
      className={css('command')}
      data-compact={compact ? 'true' : undefined}
      onClick={() => {
        onCopy(command);
      }}
    >
      <span className={css('prompt')} aria-hidden="true">
        $
      </span>
      <code className={css('command-text')}>{command}</code>
      <span className={css('copy')}>{t('web.common.copy')}</span>
    </button>
  );
}
