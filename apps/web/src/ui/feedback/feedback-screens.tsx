import { useLocale, useT } from '@ramal/ui';
import type { ReactElement } from 'react';
import { Link } from 'react-router';

import { landingPath } from '../../domain/preferences/page-preferences';
import { PRODUCT_NAME, moduleClasses } from '../shared/module-classes';

import styles from './feedback.module.css';

const css = moduleClasses(styles);

/** Lo que se ve mientras una ruta SPA carga sus datos por primera vez. */
export function LoadingScreen(): ReactElement {
  const t = useT();
  return (
    <main className={css('page')} aria-busy="true">
      <p className={css('text')}>{t('common.loading')}</p>
    </main>
  );
}

/**
 * Una ruta que no existe o un fallo inesperado al pintar. El texto no dice el
 * motivo tecnico: queda en la consola, donde ayuda a quien depura.
 */
export function ProblemScreen({ notFound }: { readonly notFound: boolean }): ReactElement {
  const t = useT();
  const locale = useLocale();
  const title = notFound ? t('web.notFound.title') : t('errors.api.unknown');
  return (
    <main className={css('page')}>
      <title>{`${title} · ${PRODUCT_NAME}`}</title>
      <h1 className={css('title')}>{title}</h1>
      {notFound ? <p className={css('text')}>{t('web.notFound.text')}</p> : null}
      <Link to={landingPath(locale)}>{t('common.home')}</Link>
    </main>
  );
}
