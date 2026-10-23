import { createTranslator } from '@git-rudder/i18n';
import type { ReactElement } from 'react';
import { type LinksFunction, type MetaFunction, useLoaderData, useLocation } from 'react-router';

import { type ReleaseLoad, loadRelease } from '../../application/use-cases/load-release';
import { landingLanguage, landingPath } from '../../domain/preferences/page-preferences';
import { DEFAULT_LANGUAGE, type Language } from '../../domain/preferences/preferences';
import { LandingScreen } from '../../ui/landing/landing-screen';
import { PRODUCT_NAME } from '../../ui/shared/module-classes';
import { container } from '../container';

/**
 * Corre en Node al construir: solo lee datos publicos, nunca nada de sesion (ADR
 * 0015). El idioma no va aqui: al prerenderizar, la peticion de datos llega a
 * `/en/_.data` y no a `/en/`, asi que se lee de la URL de la pagina.
 */
export function loader(): Promise<ReleaseLoad> {
  return loadRelease(container.releases);
}

function languageOf(pathname: string): Language {
  return landingLanguage(pathname) ?? DEFAULT_LANGUAGE;
}

const OG_LOCALES: Readonly<Record<Language, string>> = { es: 'es_ES', en: 'en_US' };

/** Titulo y descripcion en el idioma de la pagina, para buscadores y previsualizaciones. */
export const meta: MetaFunction<typeof loader> = ({ location }) => {
  const language = languageOf(location.pathname);
  const t = createTranslator(language);
  const title = `${PRODUCT_NAME}: ${t('web.hero.title')}`;
  const description = t('web.hero.subtitle');
  return [
    { title },
    { name: 'description', content: description },
    { property: 'og:title', content: title },
    { property: 'og:description', content: description },
    { property: 'og:locale', content: OG_LOCALES[language] },
  ];
};

/** Cada landing declara las dos con hreflang; la de castellano es la de por defecto. */
export const links: LinksFunction = () => [
  { rel: 'alternate', hrefLang: 'es', href: landingPath('es') },
  { rel: 'alternate', hrefLang: 'en', href: landingPath('en') },
  { rel: 'alternate', hrefLang: 'x-default', href: landingPath('es') },
];

export default function LandingRoute(): ReactElement {
  const release = useLoaderData<typeof loader>();
  const { pathname } = useLocation();
  return <LandingScreen language={languageOf(pathname)} release={release} />;
}
