// Opciones comunes de esbuild para el codigo que corre en el extension host.
//
// `vscode` queda fuera del bundle: lo pone VS Code en tiempo de ejecucion. Todo
// lo demas (zod, @ramal/i18n, @ramal/bridge) va dentro, asi la VSIX no lleva
// node_modules (`vsce package --no-dependencies`).

/** Node del Electron de VS Code: node22 vale desde la 1.138. */
const NODE_TARGET = 'node22';

/** @type {import('esbuild').BuildOptions} */
export const hostOptions = {
  bundle: true,
  platform: 'node',
  format: 'cjs',
  target: NODE_TARGET,
  external: ['vscode'],
  sourcemap: true,
  logLevel: 'info',
};
