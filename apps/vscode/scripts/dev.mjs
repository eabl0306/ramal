// Desarrollo de la extension: el extension host se recompila al guardar y los
// webviews se sirven con el servidor de Vite (recarga en caliente).
//
// Lanza la extension con la configuracion "Run Extension" de .vscode/launch.json,
// que pone GIT_RUDDER_WEBVIEW_DEV_SERVER: solo con esa variable y en modo desarrollo
// el host carga los webviews desde Vite, con su CSP de desarrollo.
import { context } from 'esbuild';
import { createServer } from 'vite';

import { hostOptions } from './esbuild-options.mjs';

const host = await context({
  ...hostOptions,
  entryPoints: ['src/extension/app/extension.ts'],
  outfile: 'dist/extension.cjs',
});
await host.watch();

const server = await createServer({ configFile: 'vite.config.ts' });
await server.listen();
server.printUrls();
