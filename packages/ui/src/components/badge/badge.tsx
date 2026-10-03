import type { ReactElement, ReactNode } from 'react';

import { useT } from '../../i18n/i18n-context';
import { Icon } from '../../icons/icon';
import { moduleClasses } from '../module-classes';

import styles from './badge.module.css';

const css = moduleClasses(styles);

/** Tonos semanticos de pildoras y puntos; cada uno es un token de color. */
export type Tone = 'accent' | 'ok' | 'warn' | 'danger' | 'info' | 'merged' | 'faint' | 'neutral';

export interface BadgeProps {
  readonly tone?: Tone;
  readonly children: ReactNode;
}

/** Pildora de texto: "Predeterminada", "Este dispositivo", el rol de un miembro. */
export function Badge({ tone = 'neutral', children }: BadgeProps): ReactElement {
  return (
    <span className={css('pill')} data-tone={tone}>
      {children}
    </span>
  );
}

export type BadgePlatform = 'github' | 'gitlab' | 'bitbucket' | 'gitea' | 'azure' | 'local';

export type PlatformBadgeSize = 14 | 18 | 20 | 24 | 26 | 30 | 34;

export interface PlatformBadgeProps {
  readonly platform: BadgePlatform;
  /**
   * Las dos letras (GH, GL, BB, GT, AZ). Las da la tabla de plataformas de
   * @git-rudder/domain, la unica fuente (gaps B-38); un repo local no las lleva.
   */
  readonly abbr?: string;
  /** Nombre accesible: la marca ("GitHub") o "Solo local". */
  readonly name: string;
  readonly size?: PlatformBadgeSize;
}

/** Cuadrado con las iniciales de la plataforma sobre su color de marca. */
export function PlatformBadge({
  platform,
  abbr,
  name,
  size = 20,
}: PlatformBadgeProps): ReactElement {
  return (
    <span
      className={css('platform')}
      data-platform={platform}
      data-size={size}
      role="img"
      aria-label={name}
    >
      {platform === 'local' ? <Icon name="local" size={Math.round(size * 0.55)} /> : abbr}
    </span>
  );
}

export type PrBadgeState = 'open' | 'draft' | 'approved' | 'changes' | 'merged' | 'closed';

/** Estado de un PR: el color sobre su tono al 14%, con el texto de @git-rudder/i18n. */
export function PrStateBadge({ state }: { readonly state: PrBadgeState }): ReactElement {
  const t = useT();
  return (
    <span className={css('pill')} data-pr-state={state}>
      {t(`repos.pr.state.${state}`)}
    </span>
  );
}

export type FileStatus = 'M' | 'A' | 'D' | 'R' | 'C';

const STATUS_KEYS = {
  M: 'diff.status.modified',
  A: 'diff.status.added',
  D: 'diff.status.deleted',
  R: 'diff.status.renamed',
  C: 'diff.status.copied',
} as const;

export interface FileStatusBadgeProps {
  readonly status: FileStatus;
  /** `letter`: la letra sola de las listas; `box`: el recuadro de la cabecera del diff. */
  readonly variant?: 'letter' | 'box';
}

/** La letra de estado de git en su color (M, A, D, R, C), con nombre accesible. */
export function FileStatusBadge({
  status,
  variant = 'letter',
}: FileStatusBadgeProps): ReactElement {
  const t = useT();
  return (
    <span
      className={css('status')}
      data-status={status}
      data-variant={variant}
      role="img"
      aria-label={t(STATUS_KEYS[status])}
    >
      {status}
    </span>
  );
}
