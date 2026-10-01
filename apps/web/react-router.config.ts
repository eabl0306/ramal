import type { Config } from '@react-router/dev/config';

import { PRERENDERED_PATHS } from './src/domain/preferences/page-preferences';

/**
 * React Router 8 en modo framework sin servidor (ADR 0015): las rutas viven en la
 * capa app del hexagono, no hay SSR en ejecucion y la landing se prerenderiza en
 * sus dos idiomas al construir. Todo lo demas lo sirve el fallback de la SPA
 * (build/client/__spa-fallback.html), que nginx devuelve para cualquier ruta que
 * no sea un fichero.
 */
export default {
  appDirectory: 'src/app',
  ssr: false,
  prerender: [...PRERENDERED_PATHS],
} satisfies Config;
