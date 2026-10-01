// Empaqueta el extension host en dist/extension.cjs, el `main` del manifiesto.
//
//   node scripts/build-host.mjs            build de produccion (minificado)
//   node scripts/build-host.mjs --watch    recompila al guardar
import { build, context } from 'esbuild';

import { hostOptions } from './esbuild-options.mjs';

const watch = process.argv.includes('--watch');

/** @type {import('esbuild').BuildOptions} */
const options = {
  ...hostOptions,
  entryPoints: ['src/extension/app/extension.ts'],
  outfile: 'dist/extension.cjs',
  minify: !watch,
};

if (watch) {
  const ctx = await context(options);
  await ctx.watch();
} else {
  await build(options);
}
