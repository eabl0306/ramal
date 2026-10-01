import { type ReactElement, type ReactNode, useMemo } from 'react';

import { useT } from '../../i18n/i18n-context';
import { moduleClasses } from '../module-classes';

import { type ToastApi, ToastContext } from './toast-context';
import styles from './toast.module.css';
import { useToastQueue } from './use-toast-queue';

const css = moduleClasses(styles);

/** 3200 ms, lo que duran los avisos del prototipo de escritorio. */
export const TOAST_DURATION = 3200;

export interface ToastProviderProps {
  readonly children: ReactNode;
  readonly duration?: number;
}

/**
 * Avisos de la esquina inferior derecha. Es una region viva propia y no el
 * Toast de React Aria porque este sigue marcado como inestable (UNSTABLE_*); un
 * aria-live educado basta para que los lectores de pantalla lean cada aviso.
 */
export function ToastProvider({
  children,
  duration = TOAST_DURATION,
}: ToastProviderProps): ReactElement {
  const t = useT();
  const { toasts, show, dismiss } = useToastQueue(duration);
  const api = useMemo<ToastApi>(() => ({ show, dismiss }), [show, dismiss]);
  return (
    <ToastContext value={api}>
      {children}
      <section className={css('region')} aria-label={t('common.notifications')}>
        <ol className={css('list')} aria-live="polite">
          {toasts.map((toast) => (
            <li key={toast.id} className={css('toast')} data-kind={toast.kind} role="status">
              <span className={css('dot')} aria-hidden="true" />
              <span>{toast.text}</span>
            </li>
          ))}
        </ol>
      </section>
    </ToastContext>
  );
}
