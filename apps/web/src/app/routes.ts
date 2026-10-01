import { type RouteConfig, index, layout, route } from '@react-router/dev/routes';

/**
 * Rutas de la web. La landing es la misma pantalla en dos URL, una por idioma,
 * y las dos se prerenderizan (react-router.config.ts). El resto es SPA: lo sirve
 * el fallback de la SPA y su idioma sale de la preferencia guardada.
 */
export default [
  layout('routes/public-layout.tsx', [
    index('routes/landing.tsx', { id: 'landing-es' }),
    route('en', 'routes/landing.tsx', { id: 'landing-en' }),
    route('login', 'routes/login.tsx'),
    route('signup', 'routes/signup.tsx'),
    route('status', 'routes/status.tsx'),
    route('*', 'routes/not-found.tsx'),
  ]),
] satisfies RouteConfig;
