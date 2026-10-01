import { useLocale, useT } from '@ramal/ui';
import type { ReactElement } from 'react';
import { Link, useLocation } from 'react-router';

import { landingLanguage, landingPath } from '../../domain/preferences/page-preferences';
import { PRODUCT_NAME, moduleClasses } from '../shared/module-classes';

import { LanguageSwitch, ThemeSwitch } from './preference-switches';
import styles from './shell.module.css';

const css = moduleClasses(styles);

/** Las secciones de la landing a las que lleva la barra, en el orden del diseno. */
const SECTIONS = [
  { id: 'features', label: 'web.nav.features' },
  { id: 'download', label: 'web.nav.download' },
  { id: 'vscode', label: 'web.nav.vscode' },
] as const;

/** La primera letra del producto hace de logotipo hasta que exista uno. */
function Logo({ to }: { readonly to: string }): ReactElement {
  return (
    <Link to={to} className={css('logo')}>
      <span className={css('logo-mark')} aria-hidden="true">
        {PRODUCT_NAME.charAt(0)}
      </span>
      <span className={css('logo-name')}>{PRODUCT_NAME}</span>
    </Link>
  );
}

/**
 * Barra publica: logotipo, secciones de la landing, idioma, tema y las entradas
 * a la cuenta. Fuera de la landing las secciones llevan a la landing del idioma
 * actual, que es una pagina prerenderizada.
 */
export function SiteHeader(): ReactElement {
  const t = useT();
  const locale = useLocale();
  const { pathname } = useLocation();
  const home = landingPath(locale);
  const onLanding = landingLanguage(pathname) !== null;

  return (
    <header className={css('header')}>
      <Logo to={home} />
      <nav className={css('nav')}>
        {SECTIONS.map((section) => (
          <a
            key={section.id}
            className={css('nav-link')}
            href={onLanding ? `#${section.id}` : `${home}#${section.id}`}
          >
            {t(section.label)}
          </a>
        ))}
      </nav>
      <div className={css('spacer')} />
      <LanguageSwitch />
      <ThemeSwitch />
      <Link to="/login" className={css('text-link')}>
        {t('web.nav.login')}
      </Link>
      <Link to="/signup" className={css('cta-link')}>
        {t('web.nav.signup')}
      </Link>
    </header>
  );
}

export function SiteFooter(): ReactElement {
  const t = useT();
  return (
    <footer className={css('footer')}>
      <span className={css('footer-brand')}>{PRODUCT_NAME}</span>
      <Link to="/status" className={css('footer-link')}>
        {t('web.footer.status')}
      </Link>
    </footer>
  );
}
