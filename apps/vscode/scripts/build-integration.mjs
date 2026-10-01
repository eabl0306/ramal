// Compila los tests de integracion a out/test/integration/*.test.cjs. Corren con
// mocha dentro del extension host de un VS Code de verdad, que carga CommonJS;
// `vscode` y `mocha` los pone ese entorno.
import { build } from 'esbuild';

import { hostOptions } from './esbuild-options.mjs';

await build({
  ...hostOptions,
  entryPoints: ['test/integration/*.test.ts'],
  outdir: 'out/test/integration',
  outExtension: { '.js': '.cjs' },
  external: [...(hostOptions.external ?? []), 'mocha'],
});
