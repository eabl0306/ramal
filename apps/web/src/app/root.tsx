import '@git-rudder/ui/themes/git-rudder.css';
import '@git-rudder/ui/themes/fonts';
import '../ui/shell/global.css';

import type { ReactElement, ReactNode } from 'react';
import {
  Links,
  Meta,
  Outlet,
  Scripts,
  ScrollRestoration,
  isRouteErrorResponse,
  useRouteError,
} from 'react-router';

import { LoadingScreen, ProblemScreen } from '../ui/feedback/feedback-screens';
import { usePageSettings } from '../ui/preferences/page-settings';

import { AppProviders } from './app-providers';
import { container } from './container';

/**
 * El documento entero. El idioma y el tema van en <html> (data-theme y lang)
 * porque los temas de @git-rudder/ui cuelgan de ahi; data-host="web" sube la letra a
 * 14 px. Se renderiza tambien al prerenderizar, con los valores por defecto.
 */
export function Layout({ children }: { readonly children: ReactNode }): ReactElement {
  const { language, theme } = usePageSettings(container.preferences, container.visitor);
  return (
    <html lang={language} data-theme={theme} data-density="comfortable" data-host="web">
      <head>
        <meta charSet="utf-8" />
        <meta name="viewport" content="width=device-width, initial-scale=1" />
        <Meta />
        <Links />
      </head>
      <body>
        <AppProviders ports={container} language={language}>
          {children}
        </AppProviders>
        <ScrollRestoration />
        <Scripts />
      </body>
    </html>
  );
}

export default function App(): ReactElement {
  return <Outlet />;
}

/** El fallback de la SPA (__spa-fallback.html) pinta esto hasta que hidrata la ruta. */
export function HydrateFallback(): ReactElement {
  return <LoadingScreen />;
}

export function ErrorBoundary(): ReactElement {
  const error = useRouteError();
  return <ProblemScreen notFound={isRouteErrorResponse(error) && error.status === 404} />;
}
